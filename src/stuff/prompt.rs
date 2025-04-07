use crate::config::config;
use crate::stuff::paper::Paper;
use std::fmt::Write;

const READY: &str = "Если Вы загрузили все фотографии, то отправьте слово: Готово";

pub struct Prompt {
    paper: Paper,
}

impl Prompt {
    pub fn new() -> Self {
        let paper = Paper::new();
        Self { paper }
    }

    pub fn try_get_paper(&self, idx: usize) -> Option<String> {
        if idx >= self.paper.paper_vec().len() {
            None
        } else {
            Some(self.paper.paper_vec()[idx].clone())
        }
    }

    pub fn try_get_size(&self, paper: &str, idx: usize) -> Option<(String, i32)> {
        let sizes = self.paper.sizes_by_paper(paper);
        if idx >= sizes.len() {
            None
        } else {
            Some((sizes[idx].size.clone(), sizes[idx].price))
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

    pub fn final_prompt(&self, order_id: String) -> String {
        format!(
            "Ваш заказ {} принят!\n\nПолучение по адресу:{}\nтел: {}",
            order_id,
            config().SHOP_ADDRESS,
            config().SHOP_PHONE
        )
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
