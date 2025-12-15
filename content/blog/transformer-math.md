---
title: "The Mathematics of Transformers: From Attention to Architecture"
date: "Jan 14th, 2026"
summary: "A deep dive into the mathematical foundations of transformer models, covering attention mechanisms, positional encodings, and the complete forward pass."
---

Transformers have revolutionized machine learning, powering everything from GPT to AlphaFold. But what's actually happening under the hood? This post breaks down the mathematics — every matrix multiplication, every normalization, every nonlinearity.

> This post assumes familiarity with linear algebra, calculus, and basic neural networks. We'll be working with matrices extensively.

---

## The Big Picture

A transformer takes a sequence of tokens and produces a sequence of contextual representations. The core innovation is **self-attention**: allowing each position to directly attend to every other position, rather than processing sequentially like RNNs.

The key components:
1. **Embeddings** — Convert tokens to vectors
2. **Positional Encoding** — Inject position information
3. **Multi-Head Self-Attention** — Model relationships between positions
4. **Feed-Forward Networks** — Process each position independently
5. **Layer Normalization** — Stabilize training
6. **Residual Connections** — Enable deep networks

Let's formalize each piece.

---

## Input Embeddings

Given a vocabulary $V$ of size $|V|$ and an embedding dimension $d_{\text{model}}$, we learn an embedding matrix:

$$E \in \mathbb{R}^{|V| \times d_{\text{model}}}$$

For an input sequence of $n$ tokens represented as one-hot vectors $x_1, \ldots, x_n$, the embedded sequence is:

$$X = \begin{bmatrix} x_1 E \\ x_2 E \\ \vdots \\ x_n E \end{bmatrix} \in \mathbb{R}^{n \times d_{\text{model}}}$$

In practice, we just index into $E$ directly. The $i$-th row of $X$ is the embedding of token $i$.

---

## Positional Encoding

Self-attention is **permutation equivariant** — it has no inherent notion of position. Without positional information, "The cat sat on the mat" and "mat the on sat cat The" would produce identical representations.

The original transformer uses sinusoidal positional encodings:

$$PE_{(pos, 2i)} = \sin\left(\frac{pos}{10000^{2i/d_{\text{model}}}}\right)$$

$$PE_{(pos, 2i+1)} = \cos\left(\frac{pos}{10000^{2i/d_{\text{model}}}}\right)$$

where $pos$ is the position in the sequence and $i$ is the dimension index.

**Why sinusoids?** The authors hypothesized that this would allow the model to learn relative positions, since for any fixed offset $k$:

$$PE_{pos+k} = f(PE_{pos})$$

can be expressed as a linear transformation. Specifically:

$$\begin{bmatrix} \sin(\omega_i (pos + k)) \\ \cos(\omega_i (pos + k)) \end{bmatrix} = \begin{bmatrix} \cos(\omega_i k) & \sin(\omega_i k) \\ -\sin(\omega_i k) & \cos(\omega_i k) \end{bmatrix} \begin{bmatrix} \sin(\omega_i pos) \\ \cos(\omega_i pos) \end{bmatrix}$$

The final input to the transformer is:

$$X' = X + PE \in \mathbb{R}^{n \times d_{\text{model}}}$$

---

## Self-Attention: The Core Mechanism

Attention answers the question: "For each position, how much should I attend to every other position?"

### Scaled Dot-Product Attention

Given an input $X \in \mathbb{R}^{n \times d_{\text{model}}}$, we project it into three spaces:

$$Q = XW^Q, \quad K = XW^K, \quad V = XW^V$$

where $W^Q, W^K \in \mathbb{R}^{d_{\text{model}} \times d_k}$ and $W^V \in \mathbb{R}^{d_{\text{model}} \times d_v}$.

The attention computation is:

$$\text{Attention}(Q, K, V) = \text{softmax}\left(\frac{QK^T}{\sqrt{d_k}}\right)V$$

Let's break this down:

**Step 1: Compute attention scores**

$$S = QK^T \in \mathbb{R}^{n \times n}$$

Each entry $S_{ij} = q_i \cdot k_j$ measures how much position $i$ should attend to position $j$.

**Step 2: Scale**

$$S' = \frac{S}{\sqrt{d_k}}$$

Without scaling, for large $d_k$, the dot products grow large, pushing softmax into regions with tiny gradients. The scaling keeps the variance of the dot products approximately 1.

**Why $\sqrt{d_k}$?** If $q$ and $k$ have components drawn i.i.d. from $\mathcal{N}(0, 1)$, then:

$$\mathbb{E}[q \cdot k] = 0, \quad \text{Var}(q \cdot k) = d_k$$

Dividing by $\sqrt{d_k}$ normalizes the variance to 1.

**Step 3: Softmax**

$$A = \text{softmax}(S') \in \mathbb{R}^{n \times n}$$

where softmax is applied row-wise:

$$A_{ij} = \frac{\exp(S'_{ij})}{\sum_{m=1}^{n} \exp(S'_{im})}$$

Each row of $A$ sums to 1 — it's a probability distribution over positions.

**Step 4: Weighted sum**

$$\text{Output} = AV \in \mathbb{R}^{n \times d_v}$$

The output at position $i$ is:

$$\text{Output}_i = \sum_{j=1}^{n} A_{ij} v_j$$

A weighted combination of all value vectors, where the weights are the attention probabilities.

---

## Multi-Head Attention

A single attention head might focus on one type of relationship. Multiple heads can capture different patterns.

For $h$ heads, we have:

$$\text{head}_i = \text{Attention}(XW_i^Q, XW_i^K, XW_i^V)$$

where $W_i^Q, W_i^K \in \mathbb{R}^{d_{\text{model}} \times d_k}$ and $W_i^V \in \mathbb{R}^{d_{\text{model}} \times d_v}$.

Typically $d_k = d_v = d_{\text{model}} / h$.

The heads are concatenated and projected:

$$\text{MultiHead}(X) = \text{Concat}(\text{head}_1, \ldots, \text{head}_h)W^O$$

where $W^O \in \mathbb{R}^{h \cdot d_v \times d_{\text{model}}}$.

The full multi-head attention output has the same shape as the input: $\mathbb{R}^{n \times d_{\text{model}}}$.

### Computational Complexity

For a sequence of length $n$ with model dimension $d$:

- Computing $QK^T$: $O(n^2 d)$
- Softmax: $O(n^2)$
- Computing $AV$: $O(n^2 d)$

**Total: $O(n^2 d)$** — quadratic in sequence length. This is the main bottleneck for long sequences.

---

## Feed-Forward Network

After attention, each position is processed independently through a two-layer MLP:

$$\text{FFN}(x) = \text{ReLU}(xW_1 + b_1)W_2 + b_2$$

where $W_1 \in \mathbb{R}^{d_{\text{model}} \times d_{ff}}$ and $W_2 \in \mathbb{R}^{d_{ff} \times d_{\text{model}}}$.

Typically $d_{ff} = 4 \cdot d_{\text{model}}$.

Modern transformers often use **GELU** instead of ReLU:

$$\text{GELU}(x) = x \cdot \Phi(x)$$

where $\Phi$ is the CDF of the standard normal distribution. An approximation:

$$\text{GELU}(x) \approx 0.5x\left(1 + \tanh\left(\sqrt{\frac{2}{\pi}}\left(x + 0.044715x^3\right)\right)\right)$$

Some architectures use **SwiGLU** (Swish-Gated Linear Unit):

$$\text{SwiGLU}(x) = (\text{Swish}(xW_1)) \odot (xW_2)$$

where $\text{Swish}(x) = x \cdot \sigma(x)$ and $\odot$ is element-wise multiplication.

---

## Layer Normalization

Normalization stabilizes training by reducing internal covariate shift.

Given a vector $x \in \mathbb{R}^d$, layer norm computes:

$$\text{LayerNorm}(x) = \gamma \odot \frac{x - \mu}{\sqrt{\sigma^2 + \epsilon}} + \beta$$

where:
- $\mu = \frac{1}{d} \sum_{i=1}^{d} x_i$ (mean)
- $\sigma^2 = \frac{1}{d} \sum_{i=1}^{d} (x_i - \mu)^2$ (variance)
- $\gamma, \beta \in \mathbb{R}^d$ are learned scale and shift parameters
- $\epsilon \approx 10^{-6}$ prevents division by zero

### Pre-Norm vs Post-Norm

**Post-Norm** (original transformer):
$$x' = \text{LayerNorm}(x + \text{SubLayer}(x))$$

**Pre-Norm** (used in GPT-2, most modern models):
$$x' = x + \text{SubLayer}(\text{LayerNorm}(x))$$

Pre-norm is more stable for deep networks — gradients flow more easily through the residual path.

---

## Residual Connections

Each sub-layer (attention, FFN) has a residual connection:

$$x' = x + f(x)$$

This allows gradients to flow directly through the network, enabling training of very deep models.

For a network with $L$ layers, without residual connections, gradients must pass through $L$ transformations. With residuals, there's always a direct path.

The gradient of the loss with respect to an early layer:

$$\frac{\partial \mathcal{L}}{\partial x_l} = \frac{\partial \mathcal{L}}{\partial x_L} \cdot \frac{\partial x_L}{\partial x_l} = \frac{\partial \mathcal{L}}{\partial x_L} \cdot \left(1 + \frac{\partial}{\partial x_l}\sum_{i=l}^{L-1} f_i(x_i)\right)$$

The "1" term ensures gradients don't vanish even if the $f_i$ gradients are small.

---

## Complete Transformer Block

Putting it all together, one transformer block (pre-norm style):

$$\begin{aligned}
x'_1 &= x + \text{MultiHead}(\text{LN}(x)) \\\\
x'_2 &= x'_1 + \text{FFN}(\text{LN}(x'_1))
\end{aligned}$$

where LN is LayerNorm.

Expanding the attention:

$$\begin{aligned}
&\text{Let } \bar{x} = \text{LN}(x) \\\\
&Q = \bar{x}W^Q, \quad K = \bar{x}W^K, \quad V = \bar{x}W^V \\\\
&A = \text{softmax}\left(\frac{QK^T}{\sqrt{d_k}}\right) \\\\
&\text{Attn} = AV \cdot W^O \\\\
&x'_1 = x + \text{Attn}
\end{aligned}$$

And the FFN:

$$\begin{aligned}
&\text{Let } \bar{x}'_1 = \text{LN}(x'_1) \\\\
&h = \text{GELU}(\bar{x}'_1 W_1 + b_1) \\\\
&\text{FFN} = hW_2 + b_2 \\\\
&x'_2 = x'_1 + \text{FFN}
\end{aligned}$$

---

## Training Objective

For language modeling, we minimize the cross-entropy loss:

$$\mathcal{L} = -\sum_{t=1}^{T} \log P(x_t | x_{<t})$$

where $P(x_t | x_{<t})$ is the model's predicted probability of the correct next token.

The output of the transformer is projected to vocabulary size:

$$\text{logits} = x_L W_{\text{vocab}} \in \mathbb{R}^{n \times |V|}$$

where $W_{\text{vocab}} \in \mathbb{R}^{d_{\text{model}} \times |V|}$.

Then softmax gives probabilities:

$$P(x_t = v | x_{<t}) = \frac{\exp(\text{logits}_{t,v})}{\sum_{v'} \exp(\text{logits}_{t,v'})}$$

### Causal Masking

For autoregressive generation, position $i$ should only attend to positions $\leq i$. We add a mask:

$$\text{Attention}(Q, K, V) = \text{softmax}\left(\frac{QK^T}{\sqrt{d_k}} + M\right)V$$

where $M_{ij} = \begin{cases} 0 & \text{if } i \geq j \\ -\infty & \text{if } i < j \end{cases}$

The $-\infty$ entries become 0 after softmax.

---

## Parameter Count

For a transformer with $L$ layers, vocabulary size $|V|$, model dimension $d$, FFN dimension $d_{ff}$, and $h$ heads:

| Component | Parameters |
|-----------|------------|
| Token embeddings | $\|V\| \cdot d$ |
| Position embeddings | $n_{\max} \cdot d$ (if learned) |
| Per-layer attention | $4d^2$ (Q, K, V, O projections) |
| Per-layer FFN | $2d \cdot d_{ff}$ |
| Per-layer LayerNorm | $4d$ (two norms, each with $\gamma$ and $\beta$) |
| Output projection | Often tied with embeddings |

**Total per layer:** $4d^2 + 2d \cdot d_{ff} + 4d$

For GPT-3 175B with $L=96$, $d=12288$, $d_{ff}=49152$, $|V|=50257$:
- Per layer: $\approx 1.8B$ parameters
- Total: $\approx 175B$ parameters

---

## Gradient Flow Analysis

Consider the gradient of the loss with respect to the attention scores:

$$\frac{\partial \mathcal{L}}{\partial S} = \frac{\partial \mathcal{L}}{\partial A} \cdot \frac{\partial A}{\partial S}$$

The Jacobian of softmax is:

$$\frac{\partial A_i}{\partial S_j} = A_i(\delta_{ij} - A_j)$$

where $\delta_{ij}$ is the Kronecker delta.

This can be rewritten as:

$$\frac{\partial A}{\partial S} = \text{diag}(A) - AA^T$$

When attention is very peaked (one position dominates), $A \approx e_k$ for some $k$, and the gradient becomes sparse. This can cause training instabilities — one reason why careful initialization and warmup matter.

---

## The Full Forward Pass

Given input tokens $t_1, \ldots, t_n$:

$$\begin{aligned}
&\textbf{Embedding: } X^{(0)} = \text{Embed}(t_{1:n}) + PE_{1:n} \\\\
&\textbf{For } l = 1 \text{ to } L: \\\\
&\quad \bar{X} = \text{LayerNorm}(X^{(l-1)}) \\\\
&\quad Q, K, V = \bar{X}W^Q_l, \bar{X}W^K_l, \bar{X}W^V_l \\\\
&\quad A = \text{softmax}\left(\frac{QK^T}{\sqrt{d_k}} + M\right) \\\\
&\quad X^{(l-0.5)} = X^{(l-1)} + \text{Concat}(A^{(1)}V^{(1)}, \ldots, A^{(h)}V^{(h)})W^O_l \\\\
&\quad \bar{X}' = \text{LayerNorm}(X^{(l-0.5)}) \\\\
&\quad X^{(l)} = X^{(l-0.5)} + \text{GELU}(\bar{X}'W_1)W_2 \\\\
&\textbf{Output: } \text{logits} = \text{LayerNorm}(X^{(L)})W_{\text{vocab}}
\end{aligned}$$

---

## Numerical Stability Tricks

### Log-Sum-Exp for Softmax

Direct softmax computation can overflow. Instead:

$$\text{softmax}(x)_i = \frac{\exp(x_i - \max(x))}{\sum_j \exp(x_j - \max(x))}$$

Subtracting the max doesn't change the result but prevents overflow.

### Mixed Precision

Modern training uses FP16/BF16 for most operations, with FP32 for:
- Loss computation
- Gradient accumulation
- LayerNorm statistics

BFloat16 is preferred because it has the same exponent range as FP32, reducing overflow/underflow risk.

---

## Conclusion

The transformer's elegance lies in its simplicity: matrix multiplications, softmax, and elementwise nonlinearities. No recurrence, no convolutions — just attention.

Key takeaways:
- Attention is $O(n^2)$ — the main bottleneck
- Scaling by $\sqrt{d_k}$ prevents gradient vanishing
- Residual connections enable deep networks
- Pre-norm is more stable than post-norm
- Multi-head attention captures diverse patterns

The math isn't magic — it's carefully designed linear algebra with well-motivated choices at every step.

---

## Further Reading

- [Attention Is All You Need](https://arxiv.org/abs/1706.03762) — The original paper
- [The Illustrated Transformer](https://jalammar.github.io/illustrated-transformer/) — Visual explanations
- [GPT-2 Paper](https://d4mucfpksywv.cloudfront.net/better-language-models/language_models_are_unsupervised_multitask_learners.pdf) — Pre-norm architecture
- [On Layer Normalization](https://arxiv.org/abs/1911.07013) — Pre-norm vs post-norm analysis
