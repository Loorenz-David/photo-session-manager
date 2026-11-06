use std::{
    io::{self, BufRead, BufReader, Read, Write},
    net::{Shutdown, TcpStream},
    time::Duration,
};
pub struct FtpClient {
    control: BufReader<TcpStream>,
}
impl FtpClient {
    pub fn connect(port: u16) -> io::Result<Self> {
        let stream = TcpStream::connect(("127.0.0.1", port))?;
        stream.set_read_timeout(Some(Duration::from_secs(15)))?;
        let mut c = Self {
            control: BufReader::new(stream),
        };
        let banner = c.reply()?;
        if !banner.starts_with("220") {
            return Err(io::Error::other(banner));
        }
        Ok(c)
    }
    pub fn reply(&mut self) -> io::Result<String> {
        let mut line = String::new();
        self.control.read_line(&mut line)?;
        Ok(line.trim_end().into())
    }
    pub fn cmd(&mut self, command: &str) -> io::Result<String> {
        self.control
            .get_mut()
            .write_all(format!("{command}\r\n").as_bytes())?;
        self.reply()
    }
    pub fn login(&mut self, user: &str, password: &str) -> io::Result<String> {
        let _ = self.cmd(&format!("USER {user}"))?;
        self.cmd(&format!("PASS {password}"))
    }
    pub fn pasv(&mut self) -> io::Result<u16> {
        let reply = self.cmd("PASV")?;
        let fields: Vec<u16> = reply
            .split(['(', ')'])
            .nth(1)
            .ok_or_else(|| io::Error::other(reply.clone()))?
            .split(',')
            .map(|s| s.parse().unwrap())
            .collect();
        Ok(fields[4] * 256 + fields[5])
    }
    pub fn open_stor(&mut self, name: &str) -> io::Result<(String, Option<TcpStream>)> {
        let port = self.pasv()?;
        let data = TcpStream::connect(("127.0.0.1", port))?;
        let reply = self.cmd(&format!("STOR {name}"))?;
        Ok((reply.clone(), reply.starts_with("150").then_some(data)))
    }
    pub fn finish_stor(&mut self, data: TcpStream) -> io::Result<String> {
        data.shutdown(Shutdown::Write)?;
        drop(data);
        self.reply()
    }
    pub fn stor(&mut self, name: &str, bytes: &[u8]) -> io::Result<String> {
        let (reply, data) = self.open_stor(name)?;
        let Some(mut data) = data else {
            return Ok(reply);
        };
        data.write_all(bytes)?;
        self.finish_stor(data)
    }
    pub fn stor_rst(&mut self, name: &str, bytes: &[u8]) -> io::Result<String> {
        let (reply, data) = self.open_stor(name)?;
        let Some(mut data) = data else {
            return Ok(reply);
        };
        data.write_all(bytes)?;
        std::thread::sleep(Duration::from_millis(300));
        let linger = libc::linger {
            l_onoff: 1,
            l_linger: 0,
        };
        use std::os::fd::AsRawFd;
        let rc = unsafe {
            libc::setsockopt(
                data.as_raw_fd(),
                libc::SOL_SOCKET,
                libc::SO_LINGER,
                (&linger as *const libc::linger).cast(),
                std::mem::size_of::<libc::linger>() as libc::socklen_t,
            )
        };
        if rc != 0 {
            return Err(io::Error::last_os_error());
        }
        drop(data);
        self.reply()
    }
    pub fn stor_resume(
        &mut self,
        name: &str,
        offset: u64,
        bytes: &[u8],
    ) -> io::Result<(String, String)> {
        let rest = self.cmd(&format!("REST {offset}"))?;
        Ok((rest, self.stor(name, bytes)?))
    }
    pub fn appe(&mut self, name: &str, bytes: &[u8]) -> io::Result<String> {
        let port = self.pasv()?;
        let mut data = TcpStream::connect(("127.0.0.1", port))?;
        let reply = self.cmd(&format!("APPE {name}"))?;
        if !reply.starts_with("150") {
            return Ok(reply);
        }
        data.write_all(bytes)?;
        self.finish_stor(data)
    }
    pub fn listing(&mut self, cmd: &str) -> io::Result<(String, String)> {
        let port = self.pasv()?;
        let mut data = TcpStream::connect(("127.0.0.1", port))?;
        let preliminary = self.cmd(cmd)?;
        let mut content = String::new();
        data.read_to_string(&mut content)?;
        Ok((preliminary, format!("{}{}", content, self.reply()?)))
    }
    pub fn noop(&mut self) -> io::Result<String> {
        self.cmd("NOOP")
    }
    pub fn close_control(self) {
        drop(self.control);
    }
}
