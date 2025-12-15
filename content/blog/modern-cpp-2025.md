---
title: "Modern C++ in 2025: C++23 and C++26 Features"
date: "Jan 16th, 2026"
summary: "A comprehensive guide to the latest C++ standards with practical examples, covering std::print, std::expected, ranges, and upcoming C++26 features."
---

C++ continues to evolve rapidly. After decades of incremental improvements, **C++23** landed with genuinely transformative features, and **C++26** is shaping up to be even more ambitious. This post covers the features I find most useful, with real code examples you can try today.

> "C++ is a language that has been through a lot of iterations, and it's still going strong." — Bjarne Stroustrup

---

## Why Care About Modern C++?

If you're still writing C++11-style code, you're missing out on:

1. **Cleaner syntax** — less boilerplate, more expressive
2. **Better safety** — `std::expected`, `std::optional`, ranges
3. **Performance** — constexpr everywhere, better optimizations
4. **Developer experience** — `std::print` alone is worth the upgrade

Let's dive in.

---

## C++23 Features

### `std::print` and `std::println`

Finally, a *sane* way to print output. No more choosing between `printf`'s type unsafety and `iostream`'s verbosity.

```cpp
#include <print>
#include <vector>
#include <string>

int main() {
    std::string name = "Josh";
    int age = 25;

    // Type-safe, format-string based printing
    std::println("Hello, {}! You are {} years old.", name, age);

    // Works with containers
    std::vector<int> nums = {1, 2, 3, 4, 5};
    std::println("Numbers: {}", nums);  // Prints: Numbers: [1, 2, 3, 4, 5]

    // Formatting options
    std::println("Pi: {:.4f}", 3.14159265);  // Pi: 3.1416
    std::println("Hex: {:#x}", 255);          // Hex: 0xff
    std::println("Binary: {:08b}", 42);       // Binary: 00101010
}
```

**Key benefits:**
- Type-safe (compile-time format string checking)
- Consistent with `std::format` from C++20
- *Way* faster than `iostream`
- Supports user-defined types via `std::formatter`

---

### `std::expected<T, E>`

Error handling without exceptions. This is **huge** for performance-critical code.

```cpp
#include <expected>
#include <string>
#include <fstream>

enum class FileError {
    NotFound,
    PermissionDenied,
    Corrupted
};

std::expected<std::string, FileError> read_config(const std::string& path) {
    std::ifstream file(path);

    if (!file) {
        return std::unexpected(FileError::NotFound);
    }

    std::string content((std::istreambuf_iterator<char>(file)),
                         std::istreambuf_iterator<char>());

    if (content.empty()) {
        return std::unexpected(FileError::Corrupted);
    }

    return content;  // Implicitly wraps in expected
}

int main() {
    auto result = read_config("settings.json");

    // Method 1: Check and access
    if (result) {
        std::println("Config: {}", *result);
    } else {
        std::println("Error code: {}", static_cast<int>(result.error()));
    }

    // Method 2: value_or with default
    std::string config = result.value_or("{}");

    // Method 3: Monadic operations (C++23)
    auto parsed = result
        .transform([](const std::string& s) { return parse_json(s); })
        .or_else([](FileError e) { return default_config(); });
}
```

#### Comparison: `optional` vs `expected`

| Feature | `std::optional<T>` | `std::expected<T, E>` |
|---------|-------------------|----------------------|
| Error info | None (just "empty") | Full error type |
| Use case | "Might not exist" | "Might fail with reason" |
| Memory overhead | `sizeof(T) + 1` | `sizeof(T) + sizeof(E)` |
| Monadic ops | C++23 | C++23 |

---

### `std::mdspan` — Multidimensional Views

Perfect for scientific computing, image processing, and game development.

```cpp
#include <mdspan>
#include <vector>
#include <print>

int main() {
    // Create a 1D buffer
    std::vector<int> data(12);
    std::iota(data.begin(), data.end(), 1);

    // View it as a 3x4 matrix
    std::mdspan matrix(data.data(), 3, 4);

    // Access with multi-dimensional indexing
    std::println("Element [1][2]: {}", matrix[1, 2]);  // Row 1, Col 2

    // Iterate over rows and columns
    for (size_t i = 0; i < matrix.extent(0); ++i) {
        for (size_t j = 0; j < matrix.extent(1); ++j) {
            std::print("{:3} ", matrix[i, j]);
        }
        std::println("");
    }

    // Output:
    //   1   2   3   4
    //   5   6   7   8
    //   9  10  11  12
}
```

**Use cases:**
- Image buffers (view as height × width × channels)
- Physics simulations (3D grids)
- Neural network tensors
- Game world chunks

---

### `std::flat_map` and `std::flat_set`

Cache-friendly associative containers. The data is stored contiguously, making iteration *fast*.

```cpp
#include <flat_map>
#include <string>
#include <print>

int main() {
    std::flat_map<std::string, int> scores = {
        {"Alice", 95},
        {"Bob", 87},
        {"Charlie", 92}
    };

    // Same API as std::map
    scores["Dave"] = 88;

    // But iteration is much faster (contiguous memory)
    for (const auto& [name, score] : scores) {
        std::println("{}: {}", name, score);
    }

    // Keys and values are stored in separate vectors
    // Great for cache locality when iterating
}
```

#### When to use `flat_map` vs `map`

| Operation | `std::map` | `std::flat_map` |
|-----------|-----------|-----------------|
| Lookup | O(log n) | O(log n) |
| Insert | O(log n) | O(n) ⚠️ |
| Iteration | Slow (pointer chasing) | **Fast** (contiguous) |
| Memory | Higher overhead | Compact |

**Rule of thumb:** Use `flat_map` when you insert once and read many times.

---

### Deducing `this`

A game-changer for reducing boilerplate. Write one function that works for `const`, non-`const`, lvalue, and rvalue versions.

```cpp
#include <print>
#include <string>

class Widget {
    std::string name_;

public:
    Widget(std::string name) : name_(std::move(name)) {}

    // OLD WAY: Four overloads!
    // std::string& get_name() & { return name_; }
    // const std::string& get_name() const& { return name_; }
    // std::string&& get_name() && { return std::move(name_); }
    // const std::string&& get_name() const&& { return std::move(name_); }

    // NEW WAY: One template with deducing this
    template<typename Self>
    auto&& get_name(this Self&& self) {
        return std::forward<Self>(self).name_;
    }

    // Also great for CRTP replacement
    template<typename Self>
    void print(this Self&& self) {
        std::println("Widget: {}", self.name_);
    }
};
```

---

### `std::generator` — Coroutine-Based Lazy Sequences

Create iterators without the boilerplate.

```cpp
#include <generator>
#include <print>

std::generator<int> fibonacci(int limit) {
    int a = 0, b = 1;
    while (a < limit) {
        co_yield a;
        auto next = a + b;
        a = b;
        b = next;
    }
}

std::generator<int> primes() {
    for (int n = 2; ; ++n) {
        bool is_prime = true;
        for (int i = 2; i * i <= n; ++i) {
            if (n % i == 0) {
                is_prime = false;
                break;
            }
        }
        if (is_prime) co_yield n;
    }
}

int main() {
    // Lazy evaluation - only computes what's needed
    for (int fib : fibonacci(100)) {
        std::println("{}", fib);
    }

    // Infinite sequence, take first 10
    int count = 0;
    for (int p : primes()) {
        std::println("Prime: {}", p);
        if (++count >= 10) break;
    }
}
```

---

### `if consteval`

Distinguish between compile-time and runtime execution.

```cpp
#include <print>

constexpr int compute(int x) {
    if consteval {
        // This branch runs at compile time
        // Can use constexpr-only operations
        return x * x;
    } else {
        // This branch runs at runtime
        // Can use runtime-only operations (I/O, etc.)
        std::println("Computing at runtime: {}", x);
        return x * x;
    }
}

int main() {
    constexpr int compile_time = compute(5);  // Uses consteval branch

    int runtime_val = 7;
    int runtime = compute(runtime_val);       // Uses else branch
}
```

---

### Multidimensional Subscript Operator

No more `matrix(i, j)` hacks — use `matrix[i, j]` directly.

```cpp
template<typename T>
class Matrix {
    std::vector<T> data_;
    size_t rows_, cols_;

public:
    Matrix(size_t r, size_t c) : data_(r * c), rows_(r), cols_(c) {}

    // C++23: Multidimensional subscript operator
    T& operator[](size_t i, size_t j) {
        return data_[i * cols_ + j];
    }

    const T& operator[](size_t i, size_t j) const {
        return data_[i * cols_ + j];
    }
};

int main() {
    Matrix<double> mat(3, 3);

    mat[0, 0] = 1.0;  // Clean syntax!
    mat[1, 1] = 1.0;
    mat[2, 2] = 1.0;

    // Identity matrix created
}
```

---

## C++26 Features (Preview)

C++26 is still in development, but several major features are on track.

### `std::execution` — Structured Async

A unified model for async operations, replacing the mess of `std::async`, `std::future`, and manual thread management.

```cpp
#include <execution>
#include <print>

// Concept: Senders and Receivers
// - Sender: Describes an async operation
// - Receiver: Handles the result

auto async_read(const std::string& path) {
    return std::execution::just(path)
        | std::execution::then([](std::string p) {
            // Simulated async file read
            return read_file_contents(p);
        })
        | std::execution::upon_error([](auto error) {
            std::println("Read failed: {}", error.what());
            return std::string{};
        });
}

int main() {
    auto result = std::this_thread::sync_wait(
        async_read("config.json")
    );

    if (result) {
        std::println("Contents: {}", *result);
    }
}
```

---

### Reflection (Expected in C++26)

Compile-time introspection of types. This will revolutionize serialization, ORMs, and debugging.

```cpp
#include <meta>

struct Person {
    std::string name;
    int age;
    std::string email;
};

// Auto-generate JSON serialization
template<typename T>
std::string to_json(const T& obj) {
    std::string result = "{";
    bool first = true;

    template for (constexpr auto member : std::meta::members_of(^T)) {
        if (!first) result += ", ";
        first = false;

        result += "\"";
        result += std::meta::name_of(member);
        result += "\": ";
        result += serialize(obj.[:member:]);
    }

    return result + "}";
}

int main() {
    Person p{"Josh", 25, "josh@example.com"};
    std::println("{}", to_json(p));
    // {"name": "Josh", "age": 25, "email": "josh@example.com"}
}
```

---

### Pattern Matching (Proposed)

Rust-style pattern matching is coming to C++.

```cpp
#include <variant>
#include <string>

using Value = std::variant<int, double, std::string, std::vector<Value>>;

std::string describe(const Value& v) {
    return inspect(v) {
        i: int => std::format("integer: {}", i),
        d: double => std::format("double: {:.2f}", d),
        s: std::string if s.empty() => "empty string",
        s: std::string => std::format("string: \"{}\"", s),
        vec: std::vector<Value> => std::format("array of {} elements", vec.size()),
        _ => "unknown"
    };
}
```

---

### Contracts (Expected in C++26)

Built-in preconditions and postconditions.

```cpp
int divide(int a, int b)
    pre(b != 0, "Divisor cannot be zero")
    post(r: a == 0 || r != 0, "Non-zero divided result")
{
    return a / b;
}

class Stack {
    std::vector<int> data_;

public:
    void push(int value)
        post(size() == old(size()) + 1)
    {
        data_.push_back(value);
    }

    int pop()
        pre(!empty(), "Cannot pop from empty stack")
        post(size() == old(size()) - 1)
    {
        int val = data_.back();
        data_.pop_back();
        return val;
    }

    bool empty() const { return data_.empty(); }
    size_t size() const { return data_.size(); }
};
```

---

## Compiler Support

Here's where each major compiler stands as of early 2026:

| Feature | GCC | Clang | MSVC |
|---------|-----|-------|------|
| **C++23** | | | |
| `std::print` | 14+ | 18+ | 17.8+ |
| `std::expected` | 12+ | 16+ | 17.4+ |
| `std::mdspan` | 14+ | 18+ | 17.8+ |
| `std::flat_map` | 14+ | 18+ | 17.10+ |
| Deducing `this` | 14+ | 18+ | 17.8+ |
| `std::generator` | 14+ | 17+ | 17.6+ |
| `if consteval` | 12+ | 14+ | 17.4+ |
| Multidim `[]` | 12+ | 15+ | 17.4+ |
| **C++26 (partial)** | | | |
| `std::execution` | 15+ | 19+ | TBD |
| Reflection | TBD | TBD | TBD |
| Contracts | TBD | TBD | TBD |

---

## Practical Migration Tips

### 1. Start with `std::print`

It's a drop-in improvement with zero risk:

```cpp
// Before
std::cout << "Value: " << x << ", Name: " << name << std::endl;

// After
std::println("Value: {}, Name: {}", x, name);
```

### 2. Replace `optional` pairs with `expected`

If you're returning `std::pair<bool, T>` or similar patterns:

```cpp
// Before
std::pair<bool, User> find_user(int id) {
    if (auto it = users.find(id); it != users.end())
        return {true, *it};
    return {false, {}};
}

// After
std::expected<User, UserError> find_user(int id) {
    if (auto it = users.find(id); it != users.end())
        return *it;
    return std::unexpected(UserError::NotFound);
}
```

### 3. Use `flat_map` for static lookups

Configuration, enum-to-string mappings, etc.:

```cpp
inline const std::flat_map<Status, std::string_view> status_names = {
    {Status::Ok, "OK"},
    {Status::Error, "Error"},
    {Status::Pending, "Pending"}
};
```

---

## Resources

- [cppreference.com](https://en.cppreference.com/) — The definitive reference
- [C++ Weekly](https://www.youtube.com/c/lefticus) — Jason Turner's excellent video series
- [Compiler Explorer](https://godbolt.org/) — Test code with different compilers
- [P2300 (std::execution)](https://wg21.link/P2300) — The executors proposal
- [P2996 (Reflection)](https://wg21.link/P2996) — The reflection proposal

---

## Conclusion

C++23 is production-ready and brings genuinely useful features. If you're starting a new project or have the luxury of upgrading, **now is a great time to adopt modern C++**.

The features I find myself using daily:
1. **`std::print`** — Every single program
2. **`std::expected`** — Any function that can fail
3. **Deducing `this`** — Eliminating const/non-const overloads
4. **`std::generator`** — Lazy iteration patterns

C++26 will take another leap forward with reflection and contracts. Start experimenting with the C++23 features today, and you'll be ready when C++26 compilers mature.

Happy coding! ~~And may your compile times be short.~~
