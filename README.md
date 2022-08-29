# regxtrie
根据文本生成正则表达式, 并合并

根据 https://zhuanlan.zhihu.com/p/473683801(https://github.com/mxnaxvex/RegexMaker) 实现, 并做些调整

### feature
```
let mut root:regxtrie::TrieNode = regxtrie::TrieNode::newc("".to_string()); // 创建字典树
root.insert("asggcckjoi"); // 插入数据
root.prune(10); // 去除噪点
root.merge(); // 升级为正则表达式
let content:String = regxtrie::TrieNode::tojson(&root).unwrap();  // 保存结果
```
```
let mut root:regxtrie::TrieNode = regxtrie::TrieNode::fromjson(&fs::read_to_string(&path).unwrap()).unwrap(); // 从json加载字典树
regxtrie::ismatch(&root, s) // 返回bool, 判断是否匹配
```

### something not good
1. 插入的字符数不能过长, 否则会报达到递归上限 (之后改成循环 or 过长改为按词创建节点?)
2. 默认字典树分支 >3 才会升级正则表达式, 初始匹配可能只是精确匹配
3. 正则表达式为单纯拼接, 未做深度合并
