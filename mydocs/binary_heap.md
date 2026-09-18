# Rust 中二叉堆的实现

## 1. 堆是什么

二叉堆（binary heap）是一棵满足两个条件的完全二叉树：

1. 形状条件：除了最后一层外，每层从左到右填满；最后一层也从左到右连续填充。
2. 堆序条件：每个父节点都不劣于它的子节点。

“不劣于”由堆类型决定：

- 最小堆：父节点小于或等于子节点，根节点是最小值。
- 最大堆：父节点大于或等于子节点，根节点是最大值。

堆不是完整排序。以最小堆为例，只保证每个节点不大于自己的孩子，不保证同一层或不同子树之间的所有元素有序。

```text
        2
      /   \
     4     3
    / \
   9   7
```

上图是合法最小堆。`4` 和 `3` 不需要比较，`9` 和 `7` 也不需要比较。

本练习的实现位于 `exercises/algorithm/algorithm9.rs`。

## 2. 为什么堆适合用数组存储

完全二叉树没有中间空位，因此可以按层序遍历顺序连续存入 `Vec`：

```text
树：              数组：
        2          下标   1  2  3  4  5
      /   \        值     2  4  3  9  7
     4     3
    / \
   9   7
```

本题故意不使用第 `0` 项，将真实节点从下标 `1` 开始保存：

```rust,ignore
items: vec![T::default()]
```

第 `0` 项是哨兵（sentinel），不代表堆中的真实元素。因此：

```rust,ignore
count == 0       // 空堆
items[1]         // 根节点；仅在 count > 0 时有效
```

这样父子关系公式更简单：

| 节点下标为 `i` | 公式 |
| --- | --- |
| 父节点 | `i / 2` |
| 左孩子 | `i * 2` |
| 右孩子 | `i * 2 + 1` |

例如，下标为 `5` 的节点：

```text
parent(5) = 5 / 2 = 2
left(5)   = 5 * 2 = 10
right(5)  = 5 * 2 + 1 = 11
```

如果数组从 `0` 开始存放真实节点，公式会变为：

```text
parent(i) = (i - 1) / 2
left(i)   = 2 * i + 1
right(i)  = 2 * i + 2
```

两种设计都正确。本题选用 1 下标，代价是要求 `T: Default` 来创建第 `0` 项哨兵。

## 3. 本题的数据结构

核心字段如下：

```rust,ignore
pub struct Heap<T>
where
    T: Default,
{
    count: usize,
    items: Vec<T>,
    comparator: fn(&T, &T) -> bool,
}
```

它们的职责是：

| 字段 | 含义 |
| --- | --- |
| `count` | 真实元素数量 |
| `items` | 保存哨兵和所有真实节点的数组 |
| `comparator` | 判断第一个元素是否应位于第二个元素上方的比较规则 |

正常情况下始终有：

```text
items.len() == count + 1
```

因为多出的 `items[0]` 是哨兵。

## 4. 比较器如何同时支持最小堆和最大堆

本题没有为最小堆和最大堆各写一份堆化代码，而是把差异封装为函数指针：

```rust,ignore
comparator: fn(&T, &T) -> bool
```

其统一语义为：

```text
comparator(a, b) == true
表示 a 的优先级高于 b，a 应该更靠近根节点。
```

创建最小堆时：

```rust,ignore
let min = Heap::<i32>::new(|a, b| a < b);
```

`2` 比 `4` 优先级高，因此 `comparator(&2, &4)` 为 `true`。

创建最大堆时：

```rust,ignore
let max = Heap::<i32>::new(|a, b| a > b);
```

`9` 比 `4` 优先级高，因此 `comparator(&9, &4)` 为 `true`。

所以同一段代码：

```rust,ignore
if comparator(child, parent) {
    swap(child, parent);
}
```

对最小堆意味着“小孩子上浮”，对最大堆意味着“大孩子上浮”。

`MinHeap::new()` 和 `MaxHeap::new()` 只是更方便的构造入口：

```rust,ignore
Heap::new(|a, b| a < b) // MinHeap
Heap::new(|a, b| a > b) // MaxHeap
```

## 5. 插入：先追加，再上浮

对堆加入元素分为两步：

1. 把新元素放在数组末尾，保持完全二叉树的形状条件。
2. 不断与父节点比较，必要时交换，直到恢复堆序条件。

对应本题的 `add`：

```rust,ignore
pub fn add(&mut self, value: T) {
    self.items.push(value);
    self.count += 1;

    let mut idx = self.count;
    while idx > 1 {
        let parent = self.parent_idx(idx);
        if !(self.comparator)(&self.items[idx], &self.items[parent]) {
            break;
        }
        self.items.swap(idx, parent);
        idx = parent;
    }
}
```

### 5.1 最小堆插入示例

在最小堆中依次插入 `4`、`2`、`9`、`1`：

插入 `4`：

```text
items = [_, 4]
```

插入 `2`，先追加到下标 `2`：

```text
items = [_, 4, 2]
```

比较 `2 < 4`，成立，交换：

```text
items = [_, 2, 4]
```

插入 `9`：

```text
items = [_, 2, 4, 9]
```

`9 < 2` 不成立，不交换。

插入 `1`，追加到下标 `4`：

```text
items = [_, 2, 4, 9, 1]
```

`parent(4) = 2`，比较 `1 < 4`，交换：

```text
items = [_, 2, 1, 9, 4]
```

继续比较新位置下标 `2` 的元素：

```text
parent(2) = 1
1 < 2，交换

items = [_, 1, 2, 9, 4]
```

现在到达根节点，插入完成。

### 5.2 为什么循环条件是 idx > 1

根节点位于下标 `1`，没有真实父节点：

```rust,ignore
while idx > 1 {
    let parent = idx / 2;
    // ...
}
```

如果允许 `idx == 1` 继续，则会访问 `items[0]`。虽然第 `0` 项存在，但它只是 `T::default()` 创建的哨兵，不应参与真实元素的比较。

## 6. 取出堆顶：用末尾元素补根，再下沉

堆顶位于 `items[1]`，是最小堆的最小值或最大堆的最大值。

取出堆顶不能直接删除数组开头，因为 `Vec::remove(1)` 会移动后面的所有元素，复杂度为 `O(n)`。正确做法是：

1. 交换根节点和末尾节点。
2. 弹出末尾节点，得到原根节点。
3. 新根可能违反堆序；与优先级更高的孩子交换，持续下沉。

本题用 `swap_remove(1)` 一次完成前两步：

```rust,ignore
let result = self.items.swap_remove(1);
self.count -= 1;
```

`swap_remove(1)` 先把最后一项放到下标 `1`，再移除旧根。因为真实节点从下标 `1` 开始，最后保留下来的 `items[0]` 哨兵不会被移除。

### 6.1 下沉时为何选择优先级更高的孩子

假设最小堆当前根为 `9`，两个孩子是 `2` 和 `4`：

```text
        9
      /   \
     2     4
```

应当让 `2` 与 `9` 交换，而不是选择 `4`。如果选择 `4`：

```text
        4
      /   \
     2     9
```

根仍然大于左孩子 `2`，堆序没有恢复。

最大堆同理，应该选择两个孩子中较大的那个。于是更准确的概念不是“最小孩子”，而是“优先级最高的孩子”。

原练习的方法名是：

```rust,ignore
fn smallest_child_idx(&self, idx: usize) -> usize
```

它在最小堆中确实返回较小孩子；但在最大堆中返回较大孩子。因此可将其语义理解为：

```text
smallest_child_idx = priority_child_idx
```

逻辑如下：

```rust,ignore
let left = self.left_child_idx(idx);
let right = self.right_child_idx(idx);

if right <= self.count && comparator(items[right], items[left]) {
    right
} else {
    left
}
```

其中：

- 右孩子不存在时，唯一的左孩子一定是优先级最高的孩子。
- 右孩子存在且优先级高于左孩子时，选择右孩子。
- 否则选择左孩子。

### 6.2 下沉循环

```rust,ignore
let mut idx = 1;
while self.children_present(idx) {
    let child = self.smallest_child_idx(idx);
    if !(self.comparator)(&self.items[child], &self.items[idx]) {
        break;
    }
    self.items.swap(idx, child);
    idx = child;
}
```

它先通过 `children_present` 确认左孩子存在：

```rust,ignore
left_child_idx(idx) <= count
```

完全二叉树不可能只有右孩子而没有左孩子，所以“有左孩子”就意味着当前节点至少有一个孩子。

之后：

1. 选出优先级更高的子节点。
2. 如果该孩子并不比当前节点优先，堆序已恢复，停止。
3. 否则交换并继续向下检查。

## 7. 完整操作示例

以最小堆 `items = [_, 1, 2, 9, 4]` 为例，调用 `next()`：

初始：

```text
        1
      /   \
     2     9
    /
   4
```

执行 `swap_remove(1)` 后，取出 `1`，最后的 `4` 补到根：

```text
        4
      /   \
     2     9
```

根的孩子中 `2` 优先级更高，交换：

```text
        2
      /   \
     4     9
```

得到：

```text
返回值：Some(1)
剩余数组：[_, 2, 4, 9]
```

下一次调用 `next()` 就会返回 `Some(2)`。

## 8. Iterator 的含义

本题为 `Heap<T>` 实现了 `Iterator`：

```rust,ignore
impl<T> Iterator for Heap<T> {
    type Item = T;

    fn next(&mut self) -> Option<T> {
        // 每次取出当前堆顶
    }
}
```

这意味着 `next()` 是破坏性操作：每取一次元素，堆的长度减少 1。

```rust,ignore
let mut heap = MinHeap::new();
heap.add(3);
heap.add(1);
heap.add(2);

assert_eq!(heap.next(), Some(1));
assert_eq!(heap.len(), 2);
assert_eq!(heap.next(), Some(2));
assert_eq!(heap.next(), Some(3));
assert_eq!(heap.next(), None);
```

也可以消费整个堆：

```rust,ignore
let mut heap = MinHeap::new();
heap.add(3);
heap.add(1);
heap.add(2);

assert_eq!(heap.collect::<Vec<_>>(), vec![1, 2, 3]);
```

对最小堆，反复 `next()` 得到升序序列；对最大堆，得到降序序列。注意这一步总耗时是 `O(n log n)`，因为调用了 `n` 次 `O(log n)` 的取顶操作。

## 9. 复杂度与限制

| 操作 | 时间复杂度 | 说明 |
| --- | --- | --- |
| `len` | `O(1)` | 直接读取 `count` |
| `is_empty` | `O(1)` | 检查 `count == 0` |
| `add` | `O(log n)` | 最多上浮树高次 |
| `next` | `O(log n)` | 最多下沉树高次 |
| 查看堆顶 | `O(1)` | 本题未实现专用 `peek`，但根位于下标 1 |
| 建堆 | 本题逐个 `add` 为 `O(n log n)` | 可进一步优化为自底向上的 `O(n)` 建堆 |

堆特别适合：

- 优先队列。
- 每次都要取当前最小或最大元素的场景。
- 堆排序。
- 图算法中的最短路径、最小生成树等需要优先队列的场景。

堆不适合按任意值快速查找。堆中寻找某个指定元素通常仍需 `O(n)`。

## 10. 与标准库 BinaryHeap 的区别

标准库已有 `std::collections::BinaryHeap<T>`，默认是最大堆：

```rust
use std::collections::BinaryHeap;

let mut heap = BinaryHeap::new();
heap.push(4);
heap.push(2);
heap.push(9);

assert_eq!(heap.pop(), Some(9));
```

要使用最小堆，通常以 `std::cmp::Reverse<T>` 包装元素：

```rust
use std::cmp::Reverse;
use std::collections::BinaryHeap;

let mut heap = BinaryHeap::new();
heap.push(Reverse(4));
heap.push(Reverse(2));
heap.push(Reverse(9));

assert_eq!(heap.pop(), Some(Reverse(2)));
```

练习中的手写实现有助于理解堆化过程；实际工程中，如果标准库的固定排序规则满足需求，应优先使用 `BinaryHeap`。本题通过函数指针比较器演示了自定义优先级规则，但函数指针不能捕获外部变量；需要捕获状态的比较逻辑时，通常应设计为泛型闭包类型或改用标准库数据结构。

## 11. 容易出错的地方

| 问题 | 后果 | 正确做法 |
| --- | --- | --- |
| 从下标 0 开始却使用 `i / 2` | 父子关系错误 | 统一采用 0 下标或 1 下标公式 |
| 插入后不执行上浮 | 根不再保证是最高优先级 | 反复比较新节点与父节点 |
| 删除根后不执行下沉 | 新根可能破坏堆序 | 反复与优先级最高的孩子交换 |
| 下沉时固定选择左孩子 | 可能不能恢复堆序 | 先比较左右孩子并选择优先级更高者 |
| 右孩子不存在时访问右下标 | 越界 panic | 先检查 `right <= count` |
| 使用 `Vec::remove(1)` 删除根 | 单次删除退化为 `O(n)` | 使用 `swap_remove(1)` |
| 对空堆访问 `items[1]` | 越界 panic | `next` 首先检查 `is_empty()` |
| 把 `items[0]` 当作真实元素 | 哨兵参与比较，逻辑混乱 | 让循环在根节点停止 |

## 12. 小结

本题的关键不在于“数组排序”，而在于始终维护两条不变式：

```text
1. items[1..=count] 表示一棵完全二叉树。
2. 每个父节点的优先级不低于其任一孩子。
```

插入只可能破坏新节点到根路径上的堆序，因此使用上浮修复。取出根只可能破坏新根到叶子路径上的堆序，因此使用下沉修复。

可以将整个实现记成：

```text
插入：末尾追加 -> 向上比较交换
取顶：末尾补根 -> 向下比较交换
最小堆：小元素优先
最大堆：大元素优先
```

相关代码：[algorithm9.rs](../exercises/algorithm/algorithm9.rs)。
