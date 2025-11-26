# Grep Implementation in Rust

A regular expression (regex) engine and grep implementation in Rust. This project was originally built as part of the [CodeCrafters Grep Challenge](https://app.codecrafters.io/courses/grep/overview) and has been extended with comprehensive regex features.

## Features

### Supported Regex Patterns
- **Literal Characters** - Match exact text: `hello`, `123`, etc.
- **Character Classes** - `\d` (digits), `\w` (word characters)
- **Character Sets** - `[abc]` (positive), `[^abc]` (negative)
- **Quantifiers**
  - `?` - Zero or one occurrence
  - `+` - One or more occurrences
- **Wildcards** - `.` matches any character
- **Anchors** - `^` (start of line), `$` (end of line)
- **Alternation Groups** - `(abc|def)` for pattern alternatives

### Pattern Processing
- **Robust Parser** - Handles complex regex syntax with proper error handling
- **Quantifier Logic** - Accurate counting and matching for repetition patterns
- **Memory Safety** - Built with Rust's ownership system for safe memory management
- **Comprehensive Testing** - Extensive unit and integration tests

### Error Handling
- **Pattern Parsing Errors** - Clear error messages for invalid regex patterns
- **Class Validation** - Proper handling of malformed character classes
- **Group Validation** - Detection of unclosed alternation groups

## Architecture

The grep engine is organized into focused modules:

- **`main.rs`** - Command-line interface and main execution pipeline
- **`pattern.rs`** - Core regex parsing and matching engine
- **`errors.rs`** - Comprehensive error types and handling
- **`lib.rs`** - Public API and module organization

## Usage

### Installation and Usage

```sh
# Install from source
git clone https://github.com/your-username/codecrafters-grep-rust
cd codecrafters-grep-rust
cargo install --path .

# Run the grep implementation
echo "test string" | your_program.sh -E "pattern"

# Or compile and run directly
cargo run -- -E "pattern" < input.txt
```

### Example Usage

```sh
# Match digits
echo "hello123world" | cargo run -- -E "\d+"
# Output: Matches

# Match with wildcards
echo "cat" | cargo run -- -E "c.t"
# Output: Matches  

# Use anchors
echo "hello" | cargo run -- -E "^hello$"
# Output: Matches

# Character sets
echo "apple" | cargo run -- -E "[aeiou]"
# Output: Matches

# Quantifiers
echo "dogs" | cargo run -- -E "dogs?"
# Output: Matches (s is optional)

# Alternation groups
echo "cat" | cargo run -- -E "(cat|dog)"
# Output: Matches
```

## Implementation Highlights

- **Zero-Copy Parsing** - Efficient string processing without unnecessary allocations
- **Iterator-Based Matching** - Stream-based pattern matching for memory efficiency
- **Rust Idioms** - Leverages Rust's type system and error handling patterns
- **Test Coverage** - Comprehensive test suite covering edge cases and complex patterns
- **Performance** - Optimized for both compilation time and runtime efficiency

## TODO - Advanced Regex Features

The following regex features are planned for future implementation:

- [ ] **Zero-or-More Quantifier** - `*` for zero or more occurrences
- [ ] **Specific Quantifiers** - `{n}`, `{n,}`, `{n,m}` for exact counts
- [ ] **Non-Greedy Quantifiers** - `*?`, `+?`, `??` for minimal matching
- [ ] **Word Boundaries** - `\b` and `\B` for word edge detection
- [ ] **Escape Sequences** - `\n`, `\t`, `\r` for whitespace matching
- [ ] **Unicode Support** - Full Unicode character class support
- [ ] **Backreferences** - `\1`, `\2` for matching previously captured groups
- [ ] **Lookahead/Lookbehind** - `(?=...)`, `(?!...)`, `(?<=...)`, `(?<!...)`
- [ ] **Case Insensitive Mode** - Flag-based case insensitive matching
