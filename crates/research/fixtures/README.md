# Research fixtures (sanitized provider captures)

Written by `psm-research fixtures` (R1, 2026-11-12) from the recordings in `~/psm-research-bench/a01/`; never edited by hand (charter rule 17). Cite as `fixtures/<provider>/<file> (RESEARCH-VERIFY-n)`. Sanitizing: test-store domain → `example.myshopify.com`, account headers and Google project numbers → placeholders, request strings over 2,000 characters cut, result lists cut to 3, third-party page bodies omitted. Every file passed a scan for the three provider keys and the shop's client id and secret.

| File | Probe | What it is | Captured |
|---|---|---|---|
| `openai/image_strict_schema_gpt-6.1-sol.json` | RESEARCH-VERIFY-1, -3, -4 | ok reply: one image + strict JSON schema; usage of this candidate model | 2026-11-12T14:43:22.555Z |
| `openai/image_strict_schema_gpt-6.1-terra.json` | RESEARCH-VERIFY-1, -3, -4 | ok reply: one image + strict JSON schema; usage of this candidate model | 2026-11-12T14:43:27.300Z |
| `openai/image_strict_schema_gpt-6-luna.json` | RESEARCH-VERIFY-1, -3, -4 | ok reply: one image + strict JSON schema; usage of this candidate model | 2026-11-12T14:43:30.628Z |
| `openai/incomplete_max_output_tokens.json` | RESEARCH-VERIFY-1, -3 | `status: incomplete`, reason max_output_tokens, usage reported | 2026-11-12T14:43:36.901Z |
| `openai/web_search.json` | RESEARCH-VERIFY-1 | web_search reply: `tool_usage.web_search.num_requests`, sources | 2026-11-12T14:43:32.460Z |
| `openai/image_and_web_search.json` | RESEARCH-VERIFY-5 | image + web_search in one request, allowed_domains = trusted list | 2026-11-12T14:45:20.195Z |
| `openai/error_invalid_key.json` | RESEARCH-VERIFY-2 | HTTP 401 invalid_api_key (a made-up key string) | 2026-11-12T14:44:32.581Z |
| `openai/error_context_length.json` | RESEARCH-VERIFY-2 | HTTP 400 context_length_exceeded (request text cut) | 2026-11-12T14:44:33.327Z |
| `openai/page_open_tool.json` | RESEARCH-VERIFY-8 | web_search open_page restricted to one domain | 2026-11-12T15:17:16.837Z |
| `serpapi_lens/search_cut_to_3.json` | RESEARCH-VERIFY-6 | Google Lens search by image_id, type=all; lists cut to 3 | 2026-11-12T14:47:38.880Z |
| `serpapi_lens/upload.json` | RESEARCH-VERIFY-7 | image upload reply (`image_id`) | 2026-11-12T14:45:56.298Z |
| `serpapi_lens/error_upload_too_large.json` | RESEARCH-VERIFY-7 | HTTP 400, file over 500 KB | 2026-11-12T14:46:04.319Z |
| `serpapi_lens/error_image_id_expired.json` | RESEARCH-VERIFY-7 | HTTP 400 "Image not found." 11.9 min after upload | 2026-11-12T14:57:50.785Z |
| `serpapi_lens/account.json` | RESEARCH-VERIFY-7 | account figures (credits) | 2026-11-12T14:51:37.970Z |
| `cloud_vision/error_billing_disabled.json` | RESEARCH-VERIFY-6 | HTTP 403 BILLING_DISABLED (no web detection reply could be captured) | 2026-11-12T14:47:23.095Z |
| `shop/hosts_query.json` | RESEARCH-VERIFY-9 | read-only Admin query on the test store: primaryDomain, myshopifyDomain; webPresences needs read_markets | 2026-11-12T14:46:26.054Z |
| `shop/token.json` | RESEARCH-VERIFY-9 | client credentials grant reply (token redacted) | 2026-11-12T14:46:25.656Z |
| `pages/shop_listing_page.json` | RESEARCH-VERIFY-8 | the owner's shop: a product page read by plain HTTP, in full | 2026-11-12T14:50:19.642Z |
| `pages/shop_429_throttled.json` | RESEARCH-VERIFY-8 | the owner's shop: HTTP 429 with Retry-After after a burst of plain fetches | 2026-11-12T15:13:28.412Z |
| `pages/auctionet_com.json` | RESEARCH-VERIFY-8 | plain HTTP GET: status, headers, final URL, sizes (no body) | 2026-11-12T15:17:16.430Z |
| `pages/auktion_sweakronan_se.json` | RESEARCH-VERIFY-8 | plain HTTP GET: status, headers, final URL, sizes (no body) | 2026-11-12T15:16:05.308Z |
| `pages/www_bukowskis_com.json` | RESEARCH-VERIFY-8 | plain HTTP GET: status, headers, final URL, sizes (no body) | 2026-11-12T15:14:45.129Z |
| `pages/www_etsy_com.json` | RESEARCH-VERIFY-8 | plain HTTP GET: status, headers, final URL, sizes (no body) | 2026-11-12T15:14:34.246Z |
| `pages/www_maleforsimports_com.json` | RESEARCH-VERIFY-8 | plain HTTP GET: status, headers, final URL, sizes (no body) | 2026-11-12T15:16:00.495Z |
| `pages/midcenturymobler_com.json` | RESEARCH-VERIFY-8 | plain HTTP GET: status, headers, final URL, sizes (no body) | 2026-11-12T15:14:50.630Z |
| `pages/www_pamono_com.json` | RESEARCH-VERIFY-8 | plain HTTP GET: status, headers, final URL, sizes (no body) | 2026-11-12T15:14:38.463Z |
| `pages/vgliving_co.json` | RESEARCH-VERIFY-8 | plain HTTP GET: status, headers, final URL, sizes (no body) | 2026-11-12T15:16:11.906Z |
| `pages/www_vntg_com.json` | RESEARCH-VERIFY-8 | plain HTTP GET: status, headers, final URL, sizes (no body) | 2026-11-12T15:15:56.492Z |
