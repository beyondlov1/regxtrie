
use std::collections::HashMap;
use regex::Regex;
use serde::{Deserialize, Serialize};

const REGX_TABLE:[[&str;5]; 3]= [["\\d","[a-z]","[A-Z]","[\\u4e00-\\u9fa5]","\\\\s",],
                                 ["\\w", "\\w", "\\w", "\\w", "."],
                                 [".",".",".",".",".",],
                                 ];


#[derive(Debug,Default)]
struct TrieInput{
    v:Vec<String>,
    cnt: i32,
}

impl TrieInput{
    pub fn new()->TrieInput{
        Default::default()
    }
}

#[derive(Debug,Default,Serialize, Deserialize)]
pub struct TrieNode{
    c:String,
    root:bool,
    end:bool,
    regxi: Option<usize>,
    regxj: usize,
    regx: String,
    cnt: i32,
    children: HashMap<String,TrieNode>,
}



impl TrieNode{
    pub fn newc(c:String)->TrieNode{
        let mut node:TrieNode = Default::default();
        node.children = HashMap::new();
        node.c = c;
        node.regx = String::from(&node.c);
        node
    }

    pub fn upgrade(&mut self){
        match self.regxi {
            None => {
                let mut i:usize = 0;
                for lvl in REGX_TABLE{
                    let mut found:bool = false;
                    let mut j:usize = 0;
                    for regstr in lvl {
                        let reg: Regex = Regex::new(regstr).unwrap();
                        if reg.is_match(&self.c) {
                            self.regxi = Some(i);
                            self.regxj = j;
                            self.regx = String::from(regstr);
                            self.c =  String::from(regstr);
                            found = true;
                            break;
                        }
                        j+=1;
                    }
                    if found{
                        break;
                    }
                    i+=1;
                }
            },
            Some(i) => {
                if i < 2{
                    self.regxi = Some(i+1);
                    self.regx = String::from(REGX_TABLE[i+1][self.regxj]);
                    self.c =  String::from(REGX_TABLE[i+1][self.regxj]);
                }
            }
        };
    }

    pub fn insert(&mut self, word: &str) {
        let mut cur = self;
        for c in word.chars() {
            let mut tmp: TrieNode = TrieNode::newc(format!("{}",c));
            let mut found: bool = false;
            let mut s = format!("{}",tmp.c);
            for _i in 0..2{
                match cur.children.get(&s){
                    Some(_child) => {
                        found = true;
                        break;
                    },
                    None => {
                        tmp.upgrade();
                        s = format!("{}",tmp.c);
                    }
                }
            }
            if !found{
                s = format!("{}",c);
                cur = cur.children.entry(format!("{}",s)).or_insert(TrieNode::newc(format!("{}",s)));
            }else{
                cur = cur.children.entry(format!("{}",s)).or_insert(tmp);
            }
            cur.cnt += 1;
        }
        cur.end = true;
        cur.children.insert("".to_string(), TrieNode::newc("".to_string()));
    }

    fn insertv(&mut self, v:&Vec<String>) {
        let mut cur = self;
        for c in v {
            cur = cur.children.entry(String::from(c)).or_insert_with(|| TrieNode::newc(format!("{}",c)));
            cur.cnt += 1;
        }
        cur.end = true;
        cur.children.insert("".to_string(), TrieNode::newc("".to_string()));
    }

    fn insertti(&mut self, ti:&TrieInput) {
        println!("insertti:{:?}", ti);
        let mut cur = self;
        for t in &ti.v {
            let node = TrieNode::newc(format!("{}",t));
            cur = cur.children.entry(String::from(t)).or_insert(node);
            cur.cnt += ti.cnt;
        }
        cur.end = true;
        cur.children.insert("".to_string(), TrieNode::newc("".to_string()));
    }


    pub fn prune(&mut self, mincnt: i32){
        let mut sumcnt:i32 = 0;
        for (_, child) in &self.children{
            sumcnt += child.cnt;
        }
        let len = self.children.len();
        if sumcnt > mincnt && len > 3{
            let mut remainsumcnt = 0;
            self.children.retain(|_, node| {
                let retained = 1.0 * node.cnt as f32 / sumcnt as f32 > 1.0/ len as f32;
                if retained{
                    remainsumcnt += node.cnt;
                }
                retained
            } );
            // 校正
            self.cnt = remainsumcnt;
            for (_, child) in &mut self.children{
                child.prune(mincnt);
            }
        }
        
    }

    pub fn merge(&mut self){
        let branchn = self.children.len();
        if branchn >= 3{
            // rebuild this
            // 0. upgrade
            for (_, node) in &mut self.children{
                node.upgrade();
            }

            // 1. collect vec
            let vs: Vec<TrieInput> = collectti(&self, TrieInput::new());

            // 2. rebuild
            self.children.clear();
            for v in &vs{
                self.insertti(v);
            }
        }
        for (_, child) in &mut self.children{
            child.merge();
        }
    }

    pub fn tojson(node: &TrieNode) -> serde_json::Result<String>{
        serde_json::to_string(node)
    }

    pub fn fromjson(s:&str) -> serde_json::Result<TrieNode>{
        serde_json::from_str(s)
    }
}

fn collect(root:&TrieNode, rootstrs:Vec<String>) -> Vec<Vec<String>> {
    let mut vs: Vec<Vec<String>> = Vec::new();
    if root.children.is_empty() {
        vs.push(rootstrs);
    }else{
        for (_, node) in &root.children{
            let mut v = Vec::new();
            for rootstr in &rootstrs{
                v.push(String::from(rootstr));
            }
            if &node.c != ""{
                v.push(String::from(&node.c));
            }
            vs.append(& mut collect(node, v));
        }
    }
    vs
}
fn collectti(root:&TrieNode, mut rootinput:TrieInput) -> Vec<TrieInput> {
    let mut vs: Vec<TrieInput> = Vec::new();
    if root.children.is_empty() {
        vs.push(rootinput);
    }else{
        // 每个child创建一个Vec, 向下传递, 最终有多少叶子节点就有多少Vec
        for (_, node) in &root.children{
            let mut v = Vec::new();

            // 继承前边传过来的Vec, 拼接后边新加的字符
            for rootstr in &rootinput.v{
                v.push(String::from(rootstr));
            }
            if &node.c != ""{
                v.push(String::from(&node.c));
                // 将队尾的cnt作为整个字符串的cnt, 不断更新
                rootinput.cnt = node.cnt;
            }
            vs.append(& mut collectti(node, TrieInput{v:v, cnt: rootinput.cnt}));
        }
    }
    vs
}

fn regexlize(root:&TrieNode)-> Vec<String>{
    let vs:Vec<Vec<String>> = collect(root, Vec::new());
    let mut result = Vec::new();
    for v in vs{
        result.push(format!("{}{}{}","^",String::from_iter(v),"$"));
    }
    result
}

pub fn ismatch(root:&TrieNode, target:&str) -> bool{
    let regstrs = regexlize(root);
    for regstr in regstrs{
        let reg: Regex = Regex::new(&regstr).unwrap();
        if reg.is_match(target) {
            return true;
        }
    }
    false
}

// 分支>3 合并分支
// 合并分支按正则表达式层级
#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn testcreate() {
        let mut root:TrieNode = TrieNode::newc(String::from(""));
        let v = vec!["abc", "a8bd我","accc","adddd","abcc","abcd","abcg","Acc","DGG","DGGB"];
        for word in &v{
            root.insert(word);
        }
        // println!("{:?}",collectti(&root, TrieInput::new()));
        // println!("{:?}", root);
        // println!("{:?}",collect(&root, Vec::new()));
        root.prune(110);
        // println!("{:?}",collect(&root, Vec::new()));
        root.merge();
        root.insert("cccccccc");
        root.insert("cccbbbb");
        // println!("{:?}",collectti(&root, TrieInput::new()));
        println!("{}",TrieNode::tojson(&root).unwrap());
        // println!("{:?}",collect(&root, Vec::new()));
        // println!("{:?}",regexlize(&root));
    }

    #[test]
    fn testcreate2() {
        let mut root:TrieNode = TrieNode::newc(String::from(""));
        let v = vec!["22", "33","444","222","2222","222","222","Acc","DGG","DGGB"];
        for word in &v{
            root.insert(word);
        }
        root.prune(10);
        root.merge();
        println!("{}",TrieNode::tojson(&root).unwrap());
        // println!("{:?}",collect(&root, Vec::new()));
        println!("{:?}",regexlize(&root));
    }

    #[test]
    fn rmatch(){
        let mut root:TrieNode = TrieNode::newc(String::from(""));
        // let v = vec!["abc", "a8bd我","accc","adddd","abcc","abcd","abcg","Acc","DGG","DGGB"];
        let v = vec!["233545", "365747657","3453243","3464575","5675868","45623423","4567467","245245","234642","23462426"];
        for word in &v{
            root.insert(word);
        }

        println!("{}", TrieNode::tojson(&root).unwrap());

        root.merge();

        println!("{}", TrieNode::tojson(&root).unwrap());

        println!("{:?}",regexlize(&root));
        println!("{}",ismatch(&root, "233598945"));

        // println!("{:?}", TrieNode::fromjson(&TrieNode::tojson(&root).unwrap()).unwrap());
        
    }
}
