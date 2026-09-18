# Rust 中的 Option、Some 与 None

## 1. 三者是什么关系

`Option<T>` 是 Rust 标准库提供的枚举类型，用于表示“一个值可能存在，也可能不存在”。其核心定义可以理解为：

```rust,ignore
enum Option<T> {
    None,
    Some(T),
}
```

这里的定义仅用于说明结构，实际代码直接使用标准库中的类型，不需要重新定义。

| 名称 | 含义 |
| --- | --- |
| `Option<T>` | 一种类型，表示“可能有一个 T 类型的值” |
| `Some(value)` | `Option` 的一个变体，表示有值，并携带 `value` |
| `None` | `Option` 的另一个变体，表示没有值 |

例如：

```rust
let present: Option<i32> = Some(5);
let absent: Option<i32> = None;

assert_eq!(present, Some(5));
assert_eq!(absent, None);
```

`present` 的类型是 `Option<i32>`，不是 `i32`，也不是所谓的 `Some` 类型。

`Some` 和 `None` 可以直接使用，是因为它们由 Rust 的预导入模块（prelude）引入当前作用域。它们的完整路径分别为 `std::option::Option::Some` 和 `std::option::Option::None`。

## 2. 为什么需要 Option

假设一个函数查找数组中的元素：

- 找到了，需要返回元素。
- 没找到，需要明确表示不存在。

不能随意用 `0` 表示不存在，因为 `0` 也可能是合法数据。

```rust
let numbers = [0, 10, 20];

assert_eq!(numbers.get(0), Some(&0));
assert_eq!(numbers.get(10), None);
```

`get()` 返回 `Option<&i32>`：

- `Some(&0)` 表示找到了一个值为 `0` 的元素，并借用它。
- `None` 表示索引超出范围，没有对应元素。

`None` 不等于 `0`、空字符串或 `false`，它表达的是“没有这个值”。

```rust
assert_ne!(Some(0), None);
assert_ne!(Some(""), None);
assert_ne!(Some(false), None);
```

普通引用 `&T` 必须指向有效的 `T`，不能用空引用表示不存在。如果需要“可能不存在的引用”，应该使用 `Option<&T>`。

## 3. Some 在表达式与模式中含义不同

同样写成 `Some(...)`，在不同位置执行的事情不同。

### 3.1 在表达式中：构造一个有值的 Option

```rust
let number = 5;
let result = Some(number);

assert_eq!(result, Some(5));
```

这里 `Some(number)` 是枚举变体构造表达式，看起来类似函数调用，但它的作用是构造 `Option` 的 `Some` 变体。

编译器可以从内部值推断出 `result` 的类型是 `Option<i32>`。

如果只有 `None`，且没有其他上下文帮助推断，通常需要标明类型：

```rust
let result: Option<i32> = None;
assert!(result.is_none());
```

### 3.2 在模式中：判断变体并绑定内部值

```rust
let result = Some(5);

match result {
    Some(number) => assert_eq!(number, 5),
    None => panic!("Expected a number"),
}
```

这里 `Some(number)` 不是创建新值，也不是调用函数，而是一个模式：

1. 判断 `result` 是否为 `Some`。
2. 如果是，把内部值绑定给变量 `number`。
3. 执行该分支。

变量名可以自行选择。`Some(root)`、`Some(node)`、`Some(value)` 中的名字都只是绑定名，不具有特殊语义。

也可以直接匹配内部的具体值：

```rust
let result = Some(5);

let description = match result {
    Some(0) => "zero",
    Some(_) => "nonzero",
    None => "missing",
};

assert_eq!(description, "nonzero");
```

其中 `_` 表示匹配该位置的任意值，但不将其绑定给变量。

## 4. 如何处理 Option

### 4.1 match：处理所有可能情况

```rust
let input = Some(10);

let output = match input {
    Some(value) => value * 2,
    None => 0,
};

assert_eq!(output, 20);
```

`match` 是表达式，因此可以把整个匹配结果赋值给变量。各分支需要产生兼容的类型，并且必须覆盖所有情况。

### 4.2 if let：只关心某一种情况

```rust
let input = Some(10);

if let Some(value) = input {
    assert_eq!(value, 10);
}
```

这里表示“如果 `input` 是 `Some`，就处理内部值”。也可以添加 `else` 来处理不匹配的情况。

### 4.3 while let：每次循环重新匹配

```rust
let mut stack = vec![1, 2, 3];
let mut output = Vec::new();

while let Some(value) = stack.pop() {
    output.push(value);
}

assert_eq!(output, vec![3, 2, 1]);
```

每轮调用 `pop()`：

- 返回 `Some(value)`：进入循环。
- 返回 `None`：结束循环。

`while let` 不会自动修改被匹配的变量。在树的查找代码中，需要在循环体内主动更新 `current`，才能沿子树继续查找。

## 5. Option 与所有权

`Option<T>` 是否拥有底层数据，取决于它里面的 `T`：

| 类型 | 内部值的含义 |
| --- | --- |
| `Option<String>` | 可能拥有一个字符串 |
| `Option<Box<Node>>` | 可能拥有一个堆上的节点 |
| `Option<&String>` | 可能持有一个字符串的共享引用 |
| `Option<&mut String>` | 可能持有一个字符串的独占可变引用 |

### 5.1 按值取出 String 会移动所有权

```rust
let name = Some(String::from("Rust"));

match name {
    Some(text) => {
        let owned: String = text;
        assert_eq!(owned, "Rust");
    }
    None => {}
}

// The String was moved out; name cannot be used as a whole here.
```

上面的模式按值绑定 `text`，内部 `String` 被移出，因此之后不能再整体使用 `name`。

不是所有匹配都会移动数据：匹配方式以及内部类型是否实现 `Copy` 都会影响结果。

```rust
let number = Some(5);

if let Some(value) = number {
    assert_eq!(value, 5);
}

assert_eq!(number, Some(5));
```

`i32` 实现了 `Copy`，`Option<i32>` 也实现了 `Copy`，这里取出整数不会导致原变量失效。

### 5.2 as_ref：借用内部值

```rust
let name = Some(String::from("Rust"));

let borrowed: Option<&String> = name.as_ref();
if let Some(text) = borrowed {
    assert_eq!(text.len(), 4);
}

assert_eq!(name.as_deref(), Some("Rust"));
```

`as_ref()` 的转换是：

```text
Option<T> --共享借用--> Option<&T>
```

它创建一个装着引用的 `Option`，不会把原来的 `Option<T>` 改成另一种类型，也不会复制或移走内部数据。

### 5.3 as_mut：可变借用内部值

```rust
let mut name = Some(String::from("Rust"));

if let Some(text) = name.as_mut() {
    text.push_str(" language");
}

assert_eq!(name.as_deref(), Some("Rust language"));
```

`text` 的类型是 `&mut String`，修改的是原来的字符串。

```text
Option<T> --可变借用--> Option<&mut T>
```

这些引用都受借用规则约束，不会延长原数据的存活时间；共享借用有效期间不能进行冲突修改，可变借用有效期间不能进行冲突访问。

## 6. as_deref 与 as_ref 有什么不同

`as_deref()` 在借用内部值后，还会通过 `Deref` 解引用一次，取得其目标类型的共享引用。

对于树中的根节点：

| 表达式 | 结果类型 |
| --- | --- |
| `self.root` 的字段类型 | `Option<Box<TreeNode<T>>>` |
| `self.root.as_ref()` | `Option<&Box<TreeNode<T>>>` |
| `self.root.as_mut()` | `Option<&mut Box<TreeNode<T>>>` |
| `self.root.as_deref()` | `Option<&TreeNode<T>>` |
| `self.root.as_deref_mut()` | `Option<&mut TreeNode<T>>` |

`Box<TreeNode<T>>` 的解引用目标是 `TreeNode<T>`；同理，`String` 的解引用目标是 `str`：

```rust
let text = Some(String::from("Rust"));
let borrowed: Option<&str> = text.as_deref();

assert_eq!(borrowed, Some("Rust"));
```

不要把 `as_deref()` 理解成“递归去掉所有包装”。它根据内部类型的 `Deref::Target` 产生一次解引用后的借用。

## 7. 结合二叉搜索树理解 Option

下面的片段来自 `exercises/algorithm/algorithm4.rs`，需要结合文件中的类型定义阅读。

### 7.1 Option 负责表示空，Box 负责间接存储和所有权

```rust,ignore
root: Option<Box<TreeNode<T>>>,
```

从内向外看：

1. `TreeNode<T>`：实际节点。
2. `Box<TreeNode<T>>`：拥有堆上的节点。
3. `Option<Box<TreeNode<T>>>`：可能拥有一个节点，也可能没有节点。

```rust,ignore
root: None
root: Some(Box::new(TreeNode::new(5)))
```

第一种表示空树，第二种表示存在一个值为 `5` 的根节点。

节点的 `left` 和 `right` 使用相同类型。递归结构需要 `Box` 这样的间接层，让编译器能够确定类型大小；仅用 `Option<TreeNode<T>>` 仍然会无限递归。`Option` 本身不会自动在堆上分配内存。

### 7.2 根节点插入

```rust,ignore
fn insert(&mut self, value: T) {
    match self.root.as_mut() {
        Some(root) => root.insert(value),
        None => self.root = Some(Box::new(TreeNode::new(value))),
    }
}
```

`self.root.as_mut()` 返回 `Option<&mut Box<TreeNode<T>>>`：

- `Some(root)`：根存在，`root` 是可变引用。方法调用自动解引用 `Box`，调用 `TreeNode::insert`。
- `None`：根不存在，创建节点并写入 `self.root`。

`as_mut()` 只是临时借用，不会取走根节点。在 `None` 分支中没有需要继续使用的内部引用，因此可以为 `self.root` 赋值。

### 7.3 当前查找位置

```rust,ignore
let mut current = self.root.as_deref();
```

`current` 的类型是 `Option<&TreeNode<T>>`：

- `Some(node)`：当前有节点可以检查。
- `None`：已经走到空子树。

`mut current` 允许重新赋值这个变量，不表示可以通过其中的共享引用修改节点。

查找值 `4` 时，假设根为 `5`，其左子节点为 `3`，而 `3` 的右子节点为 `4`：

```text
current = Some(对节点 5 的引用)
    4 < 5，转向左子树
current = Some(对节点 3 的引用)
    4 > 3，转向右子树
current = Some(对节点 4 的引用)
    4 == 4，查找成功
```

整个过程只是改变引用所指向的查找位置，不移动节点，也不复制整棵树。

### 7.4 为什么 child 赋值需要星号

```rust,ignore
let child = match value.cmp(&self.value) {
    Ordering::Less => &mut self.left,
    Ordering::Greater => &mut self.right,
    Ordering::Equal => return,
};

match child {
    Some(node) => node.insert(value),
    None => *child = Some(Box::new(TreeNode::new(value))),
}
```

`child` 的类型是 `&mut Option<Box<TreeNode<T>>>`。

匹配引用时，Rust 的匹配人体工程学规则会自动处理这里的引用层级，让 `Some(node)` 中的 `node` 成为 `&mut Box<TreeNode<T>>`，不会移出 `Box`。

两种修改写法的区别是：

```rust,ignore
self.root = Some(...); // Field access automatically dereferences self.
*child = Some(...);    // Replace the entire Option behind the reference.
```

`self.root` 通过字段访问运算符 `.` 自动解引用，等价于 `(*self).root`。`child` 则直接引用整个 `Option`，需要用 `*child` 修改其指向的值。

## 8. 常用方法及其适用场景

### 8.1 只判断有无：is_some 和 is_none

```rust
let value = Some(5);
assert!(value.is_some());
assert!(!value.is_none());
```

这两个方法借用 `Option`，返回布尔值。如果同时需要内部数据，通常直接使用 `match` 或 `if let` 更清晰。

### 8.2 无值时使用默认值：unwrap_or 和 unwrap_or_else

```rust
let missing: Option<i32> = None;
assert_eq!(missing.unwrap_or(10), 10);

let missing_name: Option<String> = None;
let name = missing_name.unwrap_or_else(|| String::from("anonymous"));
assert_eq!(name, "anonymous");
```

- `unwrap_or(default)`：调用前就会计算默认值。
- `unwrap_or_else(|| ...)`：只有遇到 `None` 才执行闭包生成默认值。

它们按值接收 `Option`；对非 `Copy` 类型，这通常会消耗原变量。

### 8.3 转换内部值：map

```rust
assert_eq!(Some(5).map(|value| value * 2), Some(10));
assert_eq!(None::<i32>.map(|value| value * 2), None);
```

`map` 在 `Some` 时执行转换，在 `None` 时保持 `None`：

```text
Option<T> --map(T -> U)--> Option<U>
```

如果不希望消耗内部数据，可以先借用：

```rust
let name = Some(String::from("Rust"));
let length = name.as_ref().map(|text| text.len());

assert_eq!(length, Some(4));
assert_eq!(name.as_deref(), Some("Rust"));
```

### 8.4 转换本身也可能没有结果：and_then

```rust
let input = Some("42");
let number = input.and_then(|text| text.parse::<i32>().ok());

assert_eq!(number, Some(42));
assert_eq!(Some("abc").and_then(|text| text.parse::<i32>().ok()), None);
```

`and_then` 中的闭包返回 `Option<U>`，最终仍然得到 `Option<U>`。如果这里改用 `map`，则会得到嵌套的 `Option<Option<i32>>`。

此处 `ok()` 将 `Result` 转成 `Option`，会丢弃解析错误的具体原因。需要保留失败原因时，应继续使用 `Result<T, E>`。

### 8.5 移走内部值并留下 None：take

```rust
let mut slot = Some(String::from("Rust"));
let taken = slot.take();

assert!(slot.is_none());
assert_eq!(taken.as_deref(), Some("Rust"));
```

`take()` 通过可变借用修改原 `Option`：把内部值移走，将原位置替换为 `None`，并返回旧的 `Option<T>`。不要求 `T` 实现 `Clone`。

### 8.6 遇到 None 就提前返回：问号运算符

```rust
fn sum_first_two(values: &[i32]) -> Option<i32> {
    let first = values.first()?;
    let second = values.get(1)?;
    first.checked_add(*second)
}

assert_eq!(sum_first_two(&[3, 4]), Some(7));
assert_eq!(sum_first_two(&[3]), None);
assert_eq!(sum_first_two(&[i32::MAX, 1]), None);
```

这里的 `?`：

- 遇到 `Some(value)`，产生内部的 `value`，继续执行。
- 遇到 `None`，立即从当前函数返回 `None`。

这个例子中函数返回 `Option`，因此可以对 `Option` 使用 `?`。普通返回 `Result` 的函数不能直接这样传播 `Option`，需要先用 `ok_or` 或 `ok_or_else` 将其转换为 `Result`。

`checked_add` 也返回 `Option`：加法溢出时返回 `None`，让整数溢出成为明确的无结果情况。

### 8.7 确定有值时取出：unwrap 和 expect

```rust
let value = Some(5);
assert_eq!(value.expect("value should have been initialized"), 5);
```

两者遇到 `None` 都会 panic。区别在于 `expect` 可以附上说明信息。

不要仅为方便而对可能为空的数据使用 `unwrap()`。更适合的选择通常是：

- 两种情况都需要处理：`match`。
- 只关心有值：`if let`。
- 无值时使用默认值：`unwrap_or` 或 `unwrap_or_else`。
- 无值时让调用者处理：返回 `Option` 并使用 `?`。

## 9. 常见误区

| 误解 | 正确理解 |
| --- | --- |
| `Some` 是一种独立类型 | `Some` 是 `Option` 的枚举变体 |
| `Some(x)` 总是在调用函数 | 表达式中构造值，模式中匹配并绑定内部数据 |
| `None` 就是整数零或空字符串 | `None` 表示没有值，`Some(0)` 和 `Some("")` 都表示有值 |
| `Option<T>` 就是一个指针 | 它是枚举，内部类型可以是整数、结构体、指针或引用等 |
| 使用 `Some` 就会分配堆内存 | `Some` 本身不会自动堆分配；本例由 `Box::new` 分配节点 |
| `as_ref()` 会复制内部数据 | 它借用内部数据并返回 `Option<&T>` |
| `mut current` 表示节点可修改 | 只表示变量可重新赋值，还要看内部是 `&T` 还是 `&mut T` |
| `match` 总会消耗原值 | 是否移动取决于匹配方式、绑定方式与 `Copy` 等因素 |
| 任意地方都能省略解引用符号 | 字段访问、方法调用等有自动处理规则，直接替换引用所指值通常仍需 `*` |

## 10. 小结与参考

可以把整篇文档概括为：

```text
Option<T>       表示可能有 T，也可能没有
Some(value)     表示有值，并携带 value
None            表示没有值

match / if let  判断变体，按模式访问内部数据
as_ref          借用内部数据
as_mut          可变借用内部数据
as_deref        借用内部数据的解引用目标
take            移走内部值，在原处留下 None
```

结合树结构时，先问“现在拿到的是什么类型”：

```text
Option<Box<Node>>       可能拥有节点
Option<&Box<Node>>     可能借用拥有节点的 Box
Option<&Node>          可能直接借用节点
&mut Option<Box<Node>> 可以修改整个节点槽位
```

相关学习材料：

- 本仓库：[引用与指针机制](references_and_pointers.md)。
- 本仓库：[二叉搜索树练习](../exercises/algorithm/algorithm4.rs)。
- 标准库：[Option 文档](https://doc.rust-lang.org/std/option/enum.Option.html)。
- Rust Book：[Option 枚举](https://doc.rust-lang.org/book/ch06-01-defining-an-enum.html#the-option-enum-and-its-advantages-over-null-values)。

本文中的独立 Rust 示例可通过以下命令验证。标记为 `ignore` 的片段是类型示意或依赖练习上下文的节选，不作为独立程序编译。

```sh
rustdoc --test mydocs/option_and_some.md
```
