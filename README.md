# 🦀 Rust Grep Clone (Codecrafters.io Challenge)

This project is my implementation of the [Codecrafters.io](https://app.codecrafters.io/courses/grep/overview) grep challenge, where the goal is to build a simplified version of the classic Unix tool grep — completely from scratch — using Rust.

The project is not just a line matcher: I went deeper into regex parsing and built a custom pattern engine that supports literals, character classes, anchors, quantifiers, groups, and alternation.

## 🚀 Features

### 🔎 Pattern Matching Engine

At the core, I implemented a **mini regex engine**.

### Supported Features

- **Literals** → match exact characters (e.g., `abc`)
- **Wildcards** → `.` matches any single character
- **Character classes**:
  - `\d` → digits  
  - `\w` → identifiers (`a-z`, `A-Z`, `_`)  
  - `[abc]` → match one of given characters  
  - `[^abc]` → match any character *except* those
- **Anchors**:
  - `^` → start of line  
  - `$` → end of line  
  - `^...$` → exact line match
- **Quantifiers**:
  - `+` → one or more  
  - `?` → zero or one
- **Alternation**:
  - `(cat|dog)` → expands into multiple sub-patterns (`cat`, `dog`)

---

I wrote a full **parser** that converts regex strings into an List of tokens, and a **recursive matcher** that evaluates input against this structure.


## 📂 File Handling

- Search through **one or multiple files**.
- **Recursive folder search** with `-r`.

## ⚙️ Command-Line Interface
```bash
 # From file(s)
 ./rusty_grep -E "pattern" file1.txt file2.txt
 # Recursive search
 ./rusty_grep -r -E "pattern" <directory>
```

## 🔢 Exit Codes

- **0** → at least one match found  
- **1** → no matches found  
- errors (invalid input, file not found, etc.)


## 🔬 How It Works

### 🧩 Pattern Parsing
- Input regex is **expanded** (handles alternation like `(cat|dog)`).  
  For example, if the pattern is `I love (cat|dog)`, it is treated as two separate patterns:  
  - `I love cat`  
  - `I love dog`
- Each pattern string is parsed into a **list of tokens**  
  (e.g., `Token::Literal`, `Token::CharClass`, `Token::GroupClass`, etc.).

### 🎯 Matching
For each input line:
- The regex engine attempts to **match the tokens recursively**.
- Supports multiple *"remaining string"* states when **quantifiers** are applied.
- If **any subpattern matches**, the line is considered a match.

### 📂 File Search
- Opens each file with a **buffered reader**.
- Passes each line through the **matcher**.
- Collects matching lines and **prints them**.

## ✅ Tests

I wrote unit tests for the parser and matcher to ensure correctness.

### Examples
- `\d` → correctly parses into a **Digit class**.
- `abc+` → parses into `a`, `b`, and `c+`.
- `(cat|dog)` → expands into two subpatterns.
- **Anchors** (`^`, `$`) → tested on multiple inputs.
