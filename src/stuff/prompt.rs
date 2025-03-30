use crate::stuff::paper::{Paper, PaperSize};
use std::fmt::Write;

const READY: &str = "Если Вы загрузили все фотографии, то отправьте слово: Готово";

const FINAL: &str = r#"Ваш заказ принят!

Получение по адресу:
г. Владивосток,
Партизанский пр-т, 16, Картинная галерея

тел: 8-(423)-244-97-34"#;

pub struct Prompt {
    paper: Paper,
}

impl Prompt {
    pub fn new() -> Self {
        let paper = Paper::new();
        Self { paper }
    }

    pub fn paper_vec(&self) -> Vec<String> {
        self.paper.paper_vec()
    }

    pub fn try_get_paper(&self, idx: usize) -> Option<String> {
        if idx < 0 || idx >= self.paper.paper_vec().len() {
            None
        } else {
            Some(self.paper.paper_vec()[idx].clone())
        }
    }

    pub fn try_get_size(&self, paper: &str, idx: usize) -> Option<String> {
        let sizes = self.paper.sizes_by_paper(paper);
        if idx < 0 || idx >= sizes.len() {
            None
        } else {
            Some(format!("{}-{}руб", sizes[idx].size, sizes[idx].price))
        }
    }

    pub fn paper_prompt(&self) -> String {
        self.paper.paper_vec().iter().enumerate().fold(
            "Выберите тип бумаги: \n".to_string(),
            |mut output, (idx, b)| {
                let _ = writeln!(output, "{} - {}", idx + 1, b);
                output
            },
        )
    }

    pub fn sizes_vec(&self, paper: &str) -> Vec<PaperSize> {
        self.paper.sizes_by_paper(paper)
    }

    pub fn size_prompt(&self, paper: &str) -> String {
        let sizes = self.paper.sizes_by_paper(paper);
        sizes.iter().enumerate().fold(
            "Выберите размер фотографий: \n".to_string(),
            |mut output, (idx, p)| {
                let _ = writeln!(output, "{} - {} {}руб/шт", idx + 1, p.size, p.price);
                output
            },
        )
    }

    pub fn ready_prompt(&self) -> String {
        READY.to_owned()
    }

    pub fn final_prompt(&self) -> String {
        FINAL.to_owned()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn paper_prompt() {
        let prompt = Prompt::new();
        let prompt_str = prompt.paper_prompt();
        println!("{}", prompt_str);
        assert!(prompt_str.len() > 0);
    }
    #[test]
    fn sizes_prompt() {
        let prompt = Prompt::new();
        let prompt_str = prompt.size_prompt("глянцевая");
        println!("{}", prompt_str);
        assert!(prompt_str.len() > 0);
    }
}
