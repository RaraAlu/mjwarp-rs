//! 最小 PTX 解析：入口、指令、f32 算术分类。

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    Fma,
    Mul,
    Add,
    Sub,
    Div,
    Sqrt,
    Rcp,
    Neg,
    Other,
}

impl Kind {
    pub fn label(self) -> &'static str {
        match self {
            Kind::Fma => "fma",
            Kind::Mul => "mul",
            Kind::Add => "add",
            Kind::Sub => "sub",
            Kind::Div => "div",
            Kind::Sqrt => "sqrt",
            Kind::Rcp => "rcp",
            Kind::Neg => "neg",
            Kind::Other => "other",
        }
    }
}

#[derive(Clone, Debug)]
pub struct Instr {
    pub opcode: String,
    pub text: String,
}

impl Instr {
    pub fn kind(&self) -> Kind {
        classify(&self.opcode)
    }

    pub fn is_f32_arith(&self) -> bool {
        self.kind() != Kind::Other && self.opcode.contains(".f32")
    }
}

fn classify(opcode: &str) -> Kind {
    // 上游 subtree_com 累积使用原子加：atom.global.add.f32 / red.global.add.f32。
    if opcode.starts_with("atom.") || opcode.starts_with("red.") {
        if opcode.contains(".add.f32") {
            return Kind::Add;
        }
        return Kind::Other;
    }
    let head = opcode.split('.').next().unwrap_or("");
    match head {
        "fma" => Kind::Fma,
        "mul" => Kind::Mul,
        "add" => Kind::Add,
        "sub" => Kind::Sub,
        "div" => Kind::Div,
        "sqrt" => Kind::Sqrt,
        "rcp" => Kind::Rcp,
        "neg" => Kind::Neg,
        _ => Kind::Other,
    }
}

#[derive(Clone, Debug)]
pub struct Entry {
    pub name: String,
    pub instrs: Vec<Instr>,
}

impl Entry {
    pub fn arith(&self) -> Vec<&Instr> {
        self.instrs.iter().filter(|i| i.is_f32_arith()).collect()
    }

    /// f32 算术指令的种类序列。
    pub fn kinds(&self) -> Vec<Kind> {
        self.arith().iter().map(|i| i.kind()).collect()
    }

    /// 完整指令文本序列（用于证据）。
    pub fn texts(&self) -> Vec<String> {
        self.arith().iter().map(|i| i.text.clone()).collect()
    }
}

#[derive(Clone, Debug)]
pub struct Program {
    pub entries: Vec<Entry>,
}

impl Program {
    pub fn parse(text: &str) -> Program {
        let mut entries = Vec::new();
        let mut current: Option<Entry> = None;
        for raw in text.lines() {
            let line = raw.trim();
            if line.is_empty() || line.starts_with("//") {
                continue;
            }
            if let Some(name) = entry_name(line) {
                if let Some(done) = current.take() {
                    entries.push(done);
                }
                current = Some(Entry {
                    name,
                    instrs: Vec::new(),
                });
                continue;
            }
            let Some(entry) = current.as_mut() else {
                continue;
            };
            if line == "{" || line == "}" || line.starts_with('.') || is_label(line) {
                continue;
            }
            if let Some(instr) = parse_instr(line) {
                entry.instrs.push(instr);
            }
        }
        if let Some(done) = current {
            entries.push(done);
        }
        Program { entries }
    }

    pub fn entry(&self, name: &str) -> Option<&Entry> {
        self.entries.iter().find(|e| e.name == name)
    }
}

fn entry_name(line: &str) -> Option<String> {
    let rest = line
        .strip_prefix(".visible .entry ")
        .or_else(|| line.strip_prefix(".entry "))
        .or_else(|| line.strip_prefix(".func "))?;
    let name: String = rest
        .chars()
        .take_while(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '$' | '.'))
        .collect();
    if name.is_empty() {
        None
    } else {
        Some(name)
    }
}

fn is_label(line: &str) -> bool {
    line.ends_with(':') && !line.contains(' ')
}

fn parse_instr(line: &str) -> Option<Instr> {
    let mut tokens = line.split_whitespace();
    let mut first = tokens.next()?;
    if first.starts_with('@') {
        first = tokens.next()?;
    }
    let opcode: String = first.trim_end_matches(|c: char| c == ';').to_string();
    if opcode.is_empty() || !opcode.starts_with(|c: char| c.is_ascii_alphabetic()) {
        return None;
    }
    Some(Instr {
        opcode,
        text: line.to_string(),
    })
}

/// 统计 f32 算术运算的融合/类型分布。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Counts {
    pub fma: usize,
    pub mul: usize,
    pub add: usize,
    pub sub: usize,
    pub div: usize,
    pub sqrt: usize,
    pub rcp: usize,
    pub neg: usize,
    pub total: usize,
}

impl Counts {
    pub fn of(instrs: &[&Instr]) -> Counts {
        let mut counts = Counts::default();
        for instr in instrs {
            match instr.kind() {
                Kind::Fma => counts.fma += 1,
                Kind::Mul => counts.mul += 1,
                Kind::Add => counts.add += 1,
                Kind::Sub => counts.sub += 1,
                Kind::Div => counts.div += 1,
                Kind::Sqrt => counts.sqrt += 1,
                Kind::Rcp => counts.rcp += 1,
                Kind::Neg => counts.neg += 1,
                Kind::Other => continue,
            }
            counts.total += 1;
        }
        counts
    }

    /// 未融合计数：每条 fma 视为一次乘法 + 一次加法。
    pub fn unfused_mul(&self) -> usize {
        self.fma + self.mul
    }

    pub fn unfused_add(&self) -> usize {
        self.fma + self.add + self.sub
    }
}
