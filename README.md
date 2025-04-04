# rotab

A lightweight Rust library implementing an IP routing table using a Trie (prefix tree) data structure. Optimized for efficiency with array-based child nodes and minimal borrowing.

The word "rotab" in Persian means Date, the fruit!

## Features
- Stores IP prefixes with associated destination addresses.
- Supports default route (`0.0.0.0/0`) and specific prefixes (e.g., `192.168.1.0/24`).
- Implements longest prefix matching for routing lookups.

## Usage

### Example
```rust
use rotab::Table;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut table = Table::new();

    // Insert a default route
    table.insert_range(
        "0.0.0.0".to_owned(),
        "255.255.255.255".to_owned(),
        "0.0.0.0".to_owned(),
    )?;

    // Insert a specific prefix
    table.insert_range(
        "192.168.1.0".to_owned(),
        "192.168.1.255".to_owned(),
        "10.0.1.1".to_owned(),
    )?;

    // Lookup an IP
    let dest = table.lookup("192.168.1.10".to_owned())?;
    println!("Destination: {:?}", dest); // Prints: Some(192.168.1.1)

    let dest = table.lookup("10.0.0.1".to_owned())?;
    println!("Destination: {:?}", dest); // Prints: Some(0.0.0.0)

    Ok(())
}

```

### Key Methods
- `Table::new()`: Creates a new routing table.
- `insert_range(start: String, end: String, dest: String)`: Adds a prefix range with a destination IP.
- `lookup(ip: String)`: Returns the destination IP for the longest matching prefix.

## Dependencies
- Rust standard library (`std`).

## TODO
- [ ] Insert via a CIDR block.
- [ ] Use a Radix Trie (PATRICIA).
- [ ] More strict IP validation.
- [ ] Add `no_std` support as an optional feature to enable use in no-std environments (e.g., embedded systems).
- [ ] Include Rust documentation (`rustdoc`) for all public APIs.

## License
[MIT License](LICENSE).
