pub fn chunk_vector<T: Clone>(og_vec: &[T], chunk_len: usize) -> Vec<Vec<T>> {
    if og_vec.is_empty() || chunk_len == 0 {
        return Vec::new();
    }
    let mut result = Vec::new();
    let mut start = 0;
    while start < og_vec.len() {
        let end = std::cmp::min(start + chunk_len, og_vec.len());
        result.push(og_vec[start..end].to_vec());
        start = end;
    }
    result
}
