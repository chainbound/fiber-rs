use ethers::types::{Address, Bytes, U256};
use hex;
use serde::Serialize;
use serde_repr::Serialize_repr;
use std::{cell::RefCell, rc::Rc};

// Sources
// * https://developerlife.com/2022/02/24/rust-non-binary-tree/

#[derive(Clone, Copy, Debug, Serialize_repr)]
#[repr(u8)]
pub enum Operator {
    AND = 1,
    OR = 2,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all(serialize = "PascalCase"))]
pub struct Filter {
    pub root: Node,
}

#[derive(Clone, Debug, Default)]
pub struct FilterBuilder {
    pub root: Option<NodeRef>,
    next: Option<NodeRef>,
    last: Option<NodeRef>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all(serialize = "PascalCase"))]
pub struct Node {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub operand: Option<FilterKV>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub operator: Option<Operator>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub children: Option<Vec<NodeRef>>,
}

type NodeRef = Rc<RefCell<Node>>;

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all(serialize = "PascalCase"))]
pub struct FilterKV {
    pub key: String,

    #[serde(with = "base64")]
    pub value: Vec<u8>,
}

// The API server only accepts base64 encoding for bytes.
mod base64 {
    use serde::Serialize;
    use serde::Serializer;

    pub fn serialize<S: Serializer>(v: &Vec<u8>, s: S) -> Result<S::Ok, S::Error> {
        let base64 = base64::encode(v);
        String::serialize(&base64, s)
    }
}

impl FilterBuilder {
    pub fn new() -> FilterBuilder {
        Self::default()
    }

    pub fn to<'a>(&'a mut self, to: &'a str) -> &'a mut FilterBuilder {
        let addr: Address = to.parse().unwrap();
        let new = Rc::new(RefCell::new(Node {
            operand: Some(FilterKV {
                key: String::from("to"),
                value: addr.as_bytes().to_vec(),
            }),
            operator: None,
            children: None,
        }));

        match &mut self.root {
            // If there's a root already, append this op to `next`'s children
            Some(_) => {
                let mut next = self.next.as_ref().unwrap().borrow_mut();
                match &mut next.children {
                    Some(children) => {
                        children.push(new);
                    }
                    None => {
                        let v = vec![new];
                        next.children = Some(v);
                    }
                }
            }
            // If no root, create it and point next to it.
            None => {
                self.root = Some(new.clone());
                self.next = Some(new);
            }
        };
        self
    }

    pub fn from<'a>(&'a mut self, from: &'a str) -> &'a mut FilterBuilder {
        let addr: Address = from.parse().unwrap();
        let new = Rc::new(RefCell::new(Node {
            operand: Some(FilterKV {
                key: String::from("from"),
                value: addr.as_bytes().to_vec(),
            }),
            operator: None,
            children: None,
        }));

        match &mut self.root {
            // If there's a root already, append this op to `next`'s children
            Some(_) => {
                let mut next = self.next.as_ref().unwrap().borrow_mut();
                match &mut next.children {
                    Some(children) => {
                        children.push(new);
                    }
                    None => {
                        let v = vec![new];
                        next.children = Some(v);
                    }
                }
            }
            // If no root, create it and point next to it.
            None => {
                self.root = Some(new.clone());
                self.next = Some(new);
            }
        };
        self
    }

    pub fn method_id<'a>(&'a mut self, id: &'a str) -> &'a mut FilterBuilder {
        let method_id: Bytes = id.parse().unwrap();
        let new = Rc::new(RefCell::new(Node {
            operand: Some(FilterKV {
                key: String::from("method"),
                value: method_id.to_vec(),
            }),
            operator: None,
            children: None,
        }));

        match &mut self.root {
            // If there's a root already, append this op to `next`'s children
            Some(_) => {
                let mut next = self.next.as_ref().unwrap().borrow_mut();
                match &mut next.children {
                    Some(children) => {
                        children.push(new);
                    }
                    None => {
                        let v = vec![new];
                        next.children = Some(v);
                    }
                }
            }
            // If no root, create it and point next to it.
            None => {
                self.root = Some(new.clone());
                self.next = Some(new);
            }
        };
        self
    }

    pub fn value(&mut self, v: U256) -> &mut FilterBuilder {
        let bytes = from_u256(v);
        let new = Rc::new(RefCell::new(Node {
            operand: Some(FilterKV {
                key: String::from("value"),
                value: bytes,
            }),
            operator: None,
            children: None,
        }));

        match &mut self.root {
            // If there's a root already, append this op to `next`'s children
            Some(_) => {
                let mut next = self.next.as_ref().unwrap().borrow_mut();
                match &mut next.children {
                    Some(children) => {
                        children.push(new);
                    }
                    None => {
                        let v = vec![new];
                        next.children = Some(v);
                    }
                }
            }
            // If no root, create it and point next to it.
            None => {
                self.root = Some(new.clone());
                self.next = Some(new);
            }
        };
        self
    }

    // Creates and AND node and enters it (i.e. anything after this will be appended)
    // as a child of this node. A reference to the last node will be saved in `last`, and you
    // can re-enter that level using `exit()`.
    pub fn and(&mut self) -> &mut FilterBuilder {
        let new = Rc::new(RefCell::new(Node {
            operand: None,
            operator: Some(Operator::AND),
            children: None,
        }));

        match &mut self.root {
            Some(_) => {
                // If there's a root already, append this op to `next`'s children
                let next = self.next.as_ref().unwrap();
                let mut next_ptr = next.borrow_mut();
                match &mut next_ptr.children {
                    Some(children) => {
                        children.push(new.clone());
                    }
                    None => {
                        let v = vec![new.clone()];
                        next_ptr.children = Some(v);
                    }
                }
            }
            // If no root, create it and point next to it.
            None => {
                self.root = Some(new.clone());
            }
        };

        self.last = self.next.clone();
        self.next = Some(new);
        self
    }

    // Creates and OR node and enters it (i.e. anything after this will be appended)
    // as a child of this node. A reference to the last node will be saved in `last`, and you
    // can re-enter that level using `exit()`.
    pub fn or(&mut self) -> &mut FilterBuilder {
        let new = Rc::new(RefCell::new(Node {
            operand: None,
            operator: Some(Operator::OR),
            children: None,
        }));

        match &mut self.root {
            Some(_) => {
                // If there's a root already, append this op to `next`'s children
                let next = self.next.as_ref().unwrap();
                let mut next_ptr = next.borrow_mut();
                match &mut next_ptr.children {
                    Some(children) => {
                        children.push(new.clone());
                    }
                    None => {
                        let v = vec![new.clone()];
                        next_ptr.children = Some(v);
                    }
                }
            }

            // If no root, create it and point next to it.
            None => {
                self.root = Some(new.clone());
            }
        };

        self.last = self.next.clone();
        self.next = Some(new);
        self
    }

    /// next tells the builder to create a child at the current `next` pointer
    /// and move there.
    pub fn exit(&mut self) -> &mut FilterBuilder {
        self.next = self.last.clone();
        self
    }

    pub fn encode(&self) -> Result<Vec<u8>, serde_json::Error> {
        let f = Filter {
            root: self.root.as_deref().unwrap().borrow().to_owned(),
        };

        serde_json::to_vec(&f)
    }

    pub fn encode_pretty(&self) -> Result<String, serde_json::Error> {
        let f = Filter {
            root: self.root.as_deref().unwrap().borrow().to_owned(),
        };

        serde_json::to_string_pretty(&f)
    }
}

fn from_u256(u: U256) -> Vec<u8> {
    let mut hex = format!("{:x}", u);
    if hex.len() % 2 != 0 {
        hex = format! {"0{}", hex};
    }

    hex::decode(hex).unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        // let val = U256::from(10000);
        let mut f = FilterBuilder::new();
        let new = f
            .to("0x7a250d5630B4cF539739dF2C5dAcb4c659F2488D")
            .or()
            .to("0x68b3465833fb72A70ecDF485E0e4C7bD8665Fc45");
        // .or()
        // .from("0x7a250d5630B4cF539739dF2C5dAcb4c659F24BCD")
        // .to("0x7a250d5630B4cF539739dF2C5dAcb4c659F2488D")
        // .exit()
        // .to("0x7a250d5630B4cF539739dF2C5dAcb4c659F2488D");
        println!("{}", new.encode_pretty().unwrap());
    }
}
