fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let seam = match psm_research_tool::http::ReqwestSeam::new() {
        Ok(s) => s,
        Err(e) => {
            eprintln!("psm-research: {e}");
            std::process::exit(1);
        }
    };
    std::process::exit(psm_research_tool::run(&args, &seam));
}
