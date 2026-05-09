use anyhow::Result;
use quick_xml::events::Event;
use quick_xml::Reader;

/// Collect `<loc>` text from a sitemap or sitemap index document.
pub fn parse_locs(xml: &[u8]) -> Result<Vec<String>> {
    let mut reader = Reader::from_reader(xml);
    reader.config_mut().trim_text(true);
    let mut buf = Vec::new();
    let mut out = Vec::new();
    loop {
        match reader.read_event_into(&mut buf)? {
            Event::Start(ref e) if e.name().as_ref() == b"loc" => {
                match reader.read_event_into(&mut buf)? {
                    Event::Text(t) => {
                        let s = t.unescape()?.into_owned();
                        if !s.is_empty() {
                            out.push(s);
                        }
                    }
                    Event::CData(t) => {
                        let s = String::from_utf8_lossy(t.as_ref()).into_owned();
                        if !s.is_empty() {
                            out.push(s);
                        }
                    }
                    _ => {}
                }
            }
            Event::Eof => break,
            _ => {}
        }
        buf.clear();
    }
    Ok(out)
}
