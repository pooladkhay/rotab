use std::{
    cell::RefCell,
    net::{AddrParseError, Ipv4Addr},
    ops::{BitAnd, BitXor},
    rc::Rc,
    str::FromStr,
};

pub struct Node {
    edges: [Option<Rc<RefCell<Node>>>; 2],
    is_terminal: bool,
    dest: Option<Ipv4Addr>,
}

impl Node {
    fn new() -> Self {
        Self {
            edges: [None, None],
            dest: None,
            is_terminal: false,
        }
    }
}

pub struct Table {
    start: Rc<RefCell<Node>>,
}

impl Table {
    pub fn new() -> Self {
        Self {
            start: Rc::new(RefCell::new(Node::new())),
        }
    }

    fn ip_to_bit_vec(ip: String) -> Result<Vec<u8>, AddrParseError> {
        let ip = Ipv4Addr::from_str(&ip)?.to_bits();

        let size = (size_of::<u32>() * 8) as u32;

        let mut bit_vec = vec![];
        for i in 0..size {
            let pow = size - i - 1;
            let b = (ip).bitand(2_u32.pow(pow)) >> pow;
            bit_vec.push(b as u8);
        }

        Ok(bit_vec)
    }

    fn prefix(&self, start: String, end: String) -> Result<Vec<u8>, AddrParseError> {
        let start_ip_bits = Ipv4Addr::from_str(&start)?.to_bits();
        let end_ip_bits = Ipv4Addr::from_str(&end)?.to_bits();

        let prefix_length = start_ip_bits.bitxor(end_ip_bits).leading_zeros();

        let mut prefix = vec![];
        for i in 0..prefix_length {
            let pow = (size_of::<u32>() * 8) as u32 - i - 1;
            let b = (start_ip_bits).bitand(2_u32.pow(pow)) >> pow;
            prefix.push(b as u8);
        }

        Ok(prefix)
    }

    pub fn insert_range(
        &mut self,
        start: String,
        end: String,
        dest: String,
    ) -> Result<(), AddrParseError> {
        let prefix = self.prefix(start, end)?;

        let mut curr_node = Rc::clone(&self.start);

        for bit in prefix {
            let node = Rc::clone(&curr_node);
            let mut node = node.borrow_mut();
            let bit_idx = bit as usize;

            if let Some(next) = &node.edges[bit_idx] {
                curr_node = Rc::clone(next);
            } else {
                let next_node = Rc::new(RefCell::new(Node::new()));
                node.edges[bit_idx] = Some(Rc::clone(&next_node));

                curr_node = next_node;
            }
        }

        let mut node = curr_node.borrow_mut();
        node.dest = Some(Ipv4Addr::from_str(&dest)?);
        node.is_terminal = true;

        Ok(())
    }

    pub fn lookup(&self, ip: String) -> Result<Option<Ipv4Addr>, AddrParseError> {
        let ip = Table::ip_to_bit_vec(ip)?;

        let mut dst = {
            let n = self.start.borrow();
            if n.is_terminal { n.dest } else { None }
        };

        let mut curr_node = Rc::clone(&self.start);

        for bit in ip {
            let node = Rc::clone(&curr_node);
            let bit_idx = bit as usize;

            if let Some(next) = &node.borrow().edges[bit_idx] {
                curr_node = Rc::clone(next);

                let next = next.borrow();
                if next.is_terminal {
                    dst = next.dest;
                }
            } else {
                break;
            }
        }

        Ok(dst)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_table() -> Table {
        Table::new()
    }

    #[test]
    fn ip_to_bit_vec() {
        let test_cases = vec![
            (
                "192.168.0.1",
                vec![
                    1, 1, 0, 0, 0, 0, 0, 0, 1, 0, 1, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                    0, 0, 0, 0, 0, 1,
                ],
            ),
            ("0.0.0.0", vec![0; 32]),
            ("255.255.255.255", vec![1; 32]),
            (
                "128.0.0.0",
                vec![
                    1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                    0, 0, 0, 0, 0, 0,
                ],
            ),
            (
                "0.0.0.1",
                vec![
                    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                    0, 0, 0, 0, 0, 1,
                ],
            ),
            (
                "10.0.0.0",
                vec![
                    0, 0, 0, 0, 1, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                    0, 0, 0, 0, 0, 0,
                ],
            ),
            (
                "127.0.0.1",
                vec![
                    0, 1, 1, 1, 1, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                    0, 0, 0, 0, 0, 1,
                ],
            ),
            (
                "1.2.3.4",
                vec![
                    0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 1, 1, 0, 0,
                    0, 0, 0, 1, 0, 0,
                ],
            ),
        ];

        for (ip, expected) in test_cases {
            let res = Table::ip_to_bit_vec(ip.to_owned()).unwrap();
            assert_eq!(res.len(), 32);
            assert_eq!(res, expected);
        }
    }

    #[test]
    fn prefix_length() {
        let test_cases = vec![
            (
                "192.168.1.1",
                "192.168.1.1",
                32,
                "11000000101010000000000100000001",
            ),
            (
                "192.168.0.0",
                "192.168.0.255",
                24,
                "110000001010100000000000",
            ),
            ("10.0.0.0", "10.0.0.255", 24, "000010100000000000000000"),
            (
                "172.16.0.0",
                "172.16.0.127",
                25,
                "1010110000010000000000000",
            ),
            (
                "192.168.1.0",
                "192.168.1.127",
                25,
                "1100000010101000000000010",
            ),
            ("10.1.0.0", "10.1.255.255", 16, "0000101000000001"),
            (
                "172.20.10.0",
                "172.20.10.31",
                27,
                "101011000001010000001010000",
            ),
            (
                "192.168.100.0",
                "192.168.100.63",
                26,
                "11000000101010000110010000",
            ),
            ("10.10.0.0", "10.10.31.255", 19, "0000101000001010000"),
            ("172.31.0.0", "172.31.15.255", 20, "10101100000111110000"),
            (
                "192.168.50.0",
                "192.168.50.15",
                28,
                "1100000010101000001100100000",
            ),
            (
                "192.168.1.1",
                "192.168.1.1",
                32,
                "11000000101010000000000100000001",
            ),
            (
                "192.168.2.0",
                "192.168.2.1",
                31,
                "1100000010101000000000100000000",
            ),
            (
                "192.168.3.0",
                "192.168.3.3",
                30,
                "110000001010100000000011000000",
            ),
            (
                "192.168.255.0",
                "192.168.255.255",
                24,
                "110000001010100011111111",
            ),
            (
                "192.168.4.0",
                "192.168.4.7",
                29,
                "11000000101010000000010000000",
            ),
            (
                "192.168.5.0",
                "192.168.5.15",
                28,
                "1100000010101000000001010000",
            ),
            ("172.20.0.0", "172.20.255.255", 16, "1010110000010100"),
            ("10.20.0.0", "10.20.1.255", 23, "00001010000101000000000"),
            ("172.30.0.0", "172.30.3.255", 22, "1010110000011110000000"),
            ("10.30.0.0", "10.30.7.255", 21, "000010100001111000000"),
            ("0.0.0.0", "255.255.255.255", 0, ""),
        ];

        let table = create_table();

        for case in test_cases {
            let prefix = table.prefix(case.0.to_owned(), case.1.to_owned()).unwrap();
            assert_eq!(prefix.len(), case.2);

            let prefix_str = prefix
                .into_iter()
                .map(|p| p.to_string())
                .collect::<Vec<String>>()
                .concat();
            assert_eq!(prefix_str, case.3)
        }
    }

    #[test]
    fn test_default_route() {
        let mut table = create_table();
        table
            .insert_range(
                "0.0.0.0".to_owned(),
                "255.255.255.255".to_owned(),
                "0.0.0.0".to_owned(),
            )
            .unwrap();
        assert_eq!(
            table
                .lookup("120.0.1.1".to_owned())
                .unwrap()
                .unwrap()
                .to_string(),
            "0.0.0.0"
        );
        assert_eq!(
            table
                .lookup("10.0.0.1".to_owned())
                .unwrap()
                .unwrap()
                .to_string(),
            "0.0.0.0"
        );
    }

    #[test]
    fn test_specific_prefixes() {
        let mut table = create_table();
        table
            .insert_range(
                "10.0.1.0".to_owned(),
                "10.0.1.255".to_owned(),
                "192.168.0.1".to_owned(),
            )
            .unwrap();
        table
            .insert_range(
                "10.0.2.0".to_owned(),
                "10.0.2.255".to_owned(),
                "192.168.0.2".to_owned(),
            )
            .unwrap();
        table
            .insert_range(
                "10.0.3.0".to_owned(),
                "10.0.3.255".to_owned(),
                "192.168.0.3".to_owned(),
            )
            .unwrap();
        assert_eq!(
            table
                .lookup("10.0.1.1".to_owned())
                .unwrap()
                .unwrap()
                .to_string(),
            "192.168.0.1"
        );
        assert_eq!(
            table
                .lookup("10.0.2.1".to_owned())
                .unwrap()
                .unwrap()
                .to_string(),
            "192.168.0.2"
        );
        assert_eq!(
            table
                .lookup("10.0.3.1".to_owned())
                .unwrap()
                .unwrap()
                .to_string(),
            "192.168.0.3"
        );
    }

    #[test]
    fn test_overlapping_prefixes() {
        let mut table = create_table();
        table
            .insert_range(
                "0.0.0.0".to_owned(),
                "127.255.255.255".to_owned(),
                "1.1.1.1".to_owned(),
            )
            .unwrap();
        table
            .insert_range(
                "128.0.0.0".to_owned(),
                "255.255.255.255".to_owned(),
                "2.2.2.2".to_owned(),
            )
            .unwrap();
        assert_eq!(
            table
                .lookup("10.0.0.1".to_owned())
                .unwrap()
                .unwrap()
                .to_string(),
            "1.1.1.1"
        );
        assert_eq!(
            table
                .lookup("192.168.1.1".to_owned())
                .unwrap()
                .unwrap()
                .to_string(),
            "2.2.2.2"
        );
    }

    #[test]
    fn test_nested_prefixes() {
        let mut table = create_table();
        table
            .insert_range(
                "10.0.0.0".to_owned(),
                "10.1.255.255".to_owned(),
                "192.168.0.0".to_owned(),
            )
            .unwrap();
        table
            .insert_range(
                "10.0.1.0".to_owned(),
                "10.0.1.255".to_owned(),
                "192.168.0.1".to_owned(),
            )
            .unwrap();
        assert_eq!(
            table
                .lookup("10.0.0.1".to_owned())
                .unwrap()
                .unwrap()
                .to_string(),
            "192.168.0.0"
        );
        assert_eq!(
            table
                .lookup("10.0.1.1".to_owned())
                .unwrap()
                .unwrap()
                .to_string(),
            "192.168.0.1"
        );
    }

    #[test]
    fn test_single_ip() {
        let mut table = create_table();
        table
            .insert_range(
                "192.168.1.1".to_owned(),
                "192.168.1.1".to_owned(),
                "192.168.1.1".to_owned(),
            )
            .unwrap();
        assert_eq!(
            table
                .lookup("192.168.1.1".to_owned())
                .unwrap()
                .unwrap()
                .to_string(),
            "192.168.1.1"
        );
        assert_eq!(table.lookup("192.168.1.2".to_owned()).unwrap(), None);
    }

    #[test]
    fn test_no_matching_prefix() {
        let table = create_table();
        assert_eq!(table.lookup("192.168.1.1".to_owned()).unwrap(), None);
    }

    #[test]
    fn test_insertion_order() {
        let mut table1 = create_table();
        table1
            .insert_range(
                "10.0.0.0".to_owned(),
                "10.1.255.255".to_owned(),
                "192.168.0.0".to_owned(),
            )
            .unwrap();
        table1
            .insert_range(
                "10.0.1.0".to_owned(),
                "10.0.1.255".to_owned(),
                "192.168.0.1".to_owned(),
            )
            .unwrap();

        let mut table2 = create_table();
        table2
            .insert_range(
                "10.0.1.0".to_owned(),
                "10.0.1.255".to_owned(),
                "192.168.0.1".to_owned(),
            )
            .unwrap();
        table2
            .insert_range(
                "10.0.0.0".to_owned(),
                "10.1.255.255".to_owned(),
                "192.168.0.0".to_owned(),
            )
            .unwrap();

        assert_eq!(
            table1
                .lookup("10.0.1.1".to_owned())
                .unwrap()
                .unwrap()
                .to_string(),
            "192.168.0.1"
        );
        assert_eq!(
            table2
                .lookup("10.0.1.1".to_owned())
                .unwrap()
                .unwrap()
                .to_string(),
            "192.168.0.1"
        );
        assert_eq!(
            table1
                .lookup("10.0.0.1".to_owned())
                .unwrap()
                .unwrap()
                .to_string(),
            "192.168.0.0"
        );
        assert_eq!(
            table2
                .lookup("10.0.0.1".to_owned())
                .unwrap()
                .unwrap()
                .to_string(),
            "192.168.0.0"
        );
    }
}
