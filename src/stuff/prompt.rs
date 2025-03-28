use std::collections::BTreeMap;
use std::fmt::Write;
use std::fs::File;
use std::io::BufRead;

const READY: &str = "Если Вы загрузили все фотографии, то отправьте слово: Готово";

const FINAL: &str = r#"Ваш заказ принят!

Получение по адресу:
г. Владивосток,
Партизанский пр-т, 16, Картинная галерея

тел: 8-(423)-244-97-34"#;

pub struct Prompt {
    paper: BTreeMap<String, Vec<String>>,
    pub paper_vec: Vec<String>,
}

impl Prompt {
    pub fn new() -> Self {
        let paper = init_paper();
        let paper_vec = paper.iter().map(|p| p.0.to_string()).collect();
        Self { paper, paper_vec }
    }

    pub fn paper_prompt(&self) -> String {
        self.paper_vec.iter().enumerate().fold(
            "Выберите тип бумаги: \n".to_string(),
            |mut output, (idx, b)| {
                let _ = writeln!(output, "{} - {}", idx + 1, b);
                output
            },
        )
    }

    pub fn size_prompt(&self, paper: &str) -> String {
        self.sizes_vec(paper).iter().enumerate().fold(
            "Выберите размер фотографий: \n".to_string(),
            |mut output, (idx, b)| {
                let _ = writeln!(output, "{} - {b}", idx + 1);
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

    pub fn sizes_vec(&self, paper: &str) -> Vec<String> {
        let s = vec![];
        self.paper.get(paper)
            .unwrap_or(&s)
            .iter()
            .map(|p| p.to_string())
            .collect()
    }
}

fn init_paper() -> BTreeMap<String, Vec<String>> {
    let lines = std::io::BufReader::new(
        File::open("paper.txt").expect("File paper.txt not found in working directory"),
    )
    .lines();

    let mut data: BTreeMap<String, Vec<String>> = BTreeMap::new();

    for line in lines.map_while(Result::ok) {
        let parts = line.split(':').collect::<Vec<&str>>();
        if parts.len() != 2 {
            panic!(
                "Ошибка формата файла paper.txt\nПример строки:\nглянцевая:10x15 - 22руб;13x18 - 30руб;15x21 - 36руб;15x23 - 40руб"
            );
        }
        let paper_name = parts[0].to_string();
        let sizes = parts[1]
            .split(";")
            .map(|s| s.to_string())
            .collect::<Vec<String>>();
        data.insert(paper_name, sizes);
    }
    data
}
