# String 和 `&str` 的区别

## 1. 先给结论

- `String` 是拥有所有权的、可增长的字符串类型，数据通常存放在堆上。
- `&str` 是字符串切片，本质上是对一段 UTF-8 字符串数据的借用视图，不拥有数据。
- 日常写函数参数时，若只需要“读字符串”，优先写 `&str`；若需要“拥有并修改字符串”，再使用 `String`。

## 2. 它们各自是什么

### `String`

`String` 可以理解成 Rust 标准库提供的“可扩容字符串容器”。

```rust
let s = String::from("hello");
```

它有三个关键特征：

- 拥有这段字符串数据的所有权
- 长度可变，可以追加、拼接、清空
- 离开作用域时会自动释放内存

例如：

```rust
let mut s = String::from("hello");
s.push_str(" world");
```

### `&str`

`&str` 是字符串切片，可以把它理解成“指向某段字符串内容的只读借用”。

```rust
let s1: &str = "hello";
```

它有两个关键特征：

- 不拥有底层数据，只是借来看
- 大小固定，不能直接修改内容

字符串字面量 `"hello"` 的类型就是 `&'static str`，因为它的数据直接存在程序的只读区域中，整个程序运行期间都有效。

## 3. 内存和所有权角度理解

最核心的区别是“谁拥有数据”。

```rust
let a = String::from("green");
let b = &a;
```

这里：

- `a` 的类型是 `String`，拥有 `"green"` 这段数据
- `b` 的类型不是 `String`，而是对 `a` 内部字符串内容的借用，通常会被当成 `&str` 使用

也就是说，`String` 像“房子的主人”，`&str` 像“房子的地址说明”。`&str` 可以看到内容，但不负责释放它。

## 4. 为什么函数参数常写成 `&str`

看这类函数：

```rust
fn is_a_color_word(attempt: &str) -> bool {
    attempt == "green" || attempt == "blue" || attempt == "red"
}
```

这里写 `&str` 的好处是参数更通用：

- 可以传字符串字面量：`is_a_color_word("green")`
- 可以传 `String` 的借用：`is_a_color_word(&word)`
- 可以传某个字符串的一部分切片

如果参数写成 `String`，调用者往往要把所有权交进去，函数的适用范围反而更窄。

## 5. 为什么 `String` 能传给 `&str`

因为 `String` 支持解引用为 `str`，Rust 会做自动解引用和借用转换。

```rust
let word = String::from("green");
is_a_color_word(&word);
```

这里 `&word` 的类型表面上是 `&String`，但 Rust 会自动把它转换成函数需要的 `&str`。

这正是 `strings2.rs` 那道题的关键点：函数签名不改，传 `&word` 就可以。

## 6. 常见转换

### `&str` 转 `String`

```rust
let s1 = "hello".to_string();
let s2 = String::from("hello");
```

### `String` 转 `&str`

```rust
let s = String::from("hello");
let slice1: &str = &s;
let slice2: &str = s.as_str();
```

### 取一部分字符串切片

```rust
let s = String::from("hello");
let part = &s[0..2];
```

但这里要注意：Rust 字符串是 UTF-8 编码，切片下标必须落在合法字符边界上，否则会运行时 panic。

## 7. 什么时候用谁

可以直接记成下面这套规则：

- 只读字符串参数：用 `&str`
- 需要拥有字符串并在函数内长期保存：用 `String`
- 需要修改、拼接、增长字符串：用 `String`
- 只是临时查看某段字符串内容：用 `&str`

## 8. 一个考试式对比表

| 对比项 | `String` | `&str` |
| --- | --- | --- |
| 是否拥有所有权 | 是 | 否 |
| 是否可增长 | 是 | 否 |
| 是否负责释放内存 | 是 | 否 |
| 常见用途 | 存储、修改、拼接 | 借用、只读参数、切片 |
| 典型创建方式 | `String::from("abc")` | `"abc"`、`&s[..]` |

## 9. 最容易混淆的一句话

`String` 是“拥有数据的字符串对象”，`&str` 是“借用数据的字符串视图”。

只要你先判断“我要不要拥有它”，大多数时候就知道该用哪一个了。
