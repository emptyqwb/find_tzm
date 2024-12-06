[<img alt="github" src="https://img.shields.io/badge/github-emptyqwb?style=for-the-badge&labelColor=555555&logo=github" height="20">](https://github.com/emptyqwb/find_tzm)
[<img alt="crates.io" src="https://img.shields.io/crates/v/find_tzm.svg?style=for-the-badge&color=fc8d62&logo=rust" height="20">](https://crates.io/crates/find_tzm)
[<img alt="docs.rs" src="https://img.shields.io/badge/docs.rs-find_tzm-66c2a5?style=for-the-badge&labelColor=555555&logo=docs.rs" height="20">](https://docs.rs/find_tzm)

# find  feature code

This repository contains scripts and instructions for building the Windows kernel using the Rust programming language.

## Prerequisites

- Install Rust and Cargo: https://rustup.rs/


## Build Instructions

1. Clone this repository:
```bash
cargo add find_tzm <version>
```
# Example
```rust
fn main() {
            let mut ret_list = Vec::<u64>::new();
        let mut buffer = vec![0u8; 0x1400fffff];
        buffer[24000000-1] = 0x7b;
        buffer[24000000] = 0x48;
        buffer[24000001] = 0x8b;
        buffer[24000002] = 0x31;
        buffer[24000003] = 0x1c;
        buffer[24000004] = 0x15;
        buffer[24000005] = 0x00;
        buffer[24000006] = 0x48;
        buffer[24000007] = 0x8b;

        buffer[34000000-1] = 0x7b;
        buffer[34000000] = 0x48;
        buffer[34000001] = 0x8b;
        buffer[34000002] = 0x31;
        buffer[34000003] = 0x1c;
        buffer[34000004] = 0x15;
        buffer[34000005] = 0x00;
        buffer[34000006] = 0x48;
        buffer[34000007] = 0x8b;
        // buffer[20] = 0x48;
        // buffer[21] = 0x8b;
        // buffer[22] = 0x31;
        // buffer[23] = 0x1c;
        // buffer[24] = 0x15;
        // buffer[25] = 0x00;
        // buffer[26] = 0x48;
        // buffer[27] = 0x8b;
        let search_start_addr = buffer.as_mut_ptr() as u64;
        let search_size =  84000000;
        let tzm = "?b 48 8b 3? ?c ?? ?? 48 8b ?? ?? ??";
        let offset_size = 0;
        let search_num = 2;
        sse2_pattern_find(&mut ret_list, search_start_addr, search_size, tzm, offset_size, search_num).unwrap();
        assert_eq!(ret_list,  [24000000-1, 34000000-1]);
}
```

## MORE
see https://docs.rs/
