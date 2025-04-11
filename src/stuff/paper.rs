use serde::Deserialize;
use std::fs::File;
use std::io::Read;

#[derive(Debug, Clone, Eq, PartialEq, Deserialize)]
pub struct PaperSize {
    pub size: String,
    pub price: i32,
}

#[derive(Debug, Clone, Eq, PartialEq, Deserialize)]
pub struct PaperType {
    pub name: String,
    pub sizes: Vec<PaperSize>,
}

#[derive(Clone)]
pub struct Paper {
    paper: Vec<PaperType>,
}
impl Paper {
    pub fn new() -> Self {
        let paper = Paper::load_from_file();
        Paper { paper }
    }

    fn load_from_file() -> Vec<PaperType> {
        let mut file = File::open("paper.json").expect("paper.json not found");
        let mut buffer = String::new();
        file.read_to_string(&mut buffer)
            .expect("paper.json read error");
        serde_json::from_str::<Vec<PaperType>>(&buffer).expect("paper.json decode error")
    }

    pub fn paper_vec(&self) -> Vec<String> {
        self.paper.iter().map(|p| p.name.clone()).collect()
    }

    pub fn sizes_by_paper(&self, paper: &str) -> Vec<PaperSize> {
        let opt = self.paper.iter().find(|i| i.name.eq(paper)).cloned();
        if let Some(i) = opt { i.sizes } else { vec![] }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[tokio::test]
    async fn get_paper_test() {
        let paper = Paper::new();
        assert_eq!(paper.paper.len(), 3);
    }
}
