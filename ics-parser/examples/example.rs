const ICS: &str = include_str!("../tests/data/ntust.ics");

fn split_string_by_chunks(input: &str, chunk_size: usize) -> Vec<String> {
    // 使用 chunks 和 .collect() 將字串切割並轉換為 Vec<String>
    input
        .chars() // 將字串轉換為字元迭代器，避免多位元字元錯誤
        .collect::<Vec<_>>() // 收集字元成向量
        .chunks(chunk_size) // 每 chunk_size 個字元切割為一段
        .map(|chunk| chunk.iter().collect::<String>()) // 將每段字元重新組合為 String
        .collect() // 收集成 Vec<String>
}

fn main() {
    let data = split_string_by_chunks(ICS, 256);

    let mut parser = ics_parser::IcsParser::new(None, None);
    for chunk in data {
        parser.parse_ics_chunk(&chunk);
    }
    let events = parser.get_events();

    println!("{:#?}", events);
}
