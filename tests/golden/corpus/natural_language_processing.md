---
tags:
  - data-and-ai:nlp
  - data-and-ai:machine-learning
  - data-and-ai:deep-learning
  - data-and-ai:llm
  - concepts:embeddings
level: advanced
category: ai
duration_hours: 64
audience:
  - audiences:developers
  - audiences:data-scientists
  - audiences:architects
---

<!-- course: natural_language_processing -->
# Natural Language Processing

## Description
This course is a comprehensive, language-agnostic treatment of `Natural Language Processing` from
foundations to modern transformer architectures. It covers the linguistic concepts that `make` text
processing hard, the classical algorithms that established the field, the statistical and neural
methods that revolutionized it, and the large-language-model era that defines its current shape.
Participants will study tokenization and morphology, classical and distributional semantics, syntactic
and dependency parsing, language models from `n`-grams to transformers, sequence labeling and tagging,
text classification and sentiment analysis, named entity recognition, machine translation, question
answering, summarization, dialogue systems, retrieval-augmented generation, evaluation methodology,
and the practical concerns of building NLP systems in production. The course emphasizes the *why*
behind each technique alongside the *how*, equipping participants to choose the right tool for the
problem and to understand new techniques as they emerge.

## Duration
64 hours / 8 days

## Intended Audience
* software developers building text-driven applications
* data scientists working with textual data
* `machine learning` engineers extending models to new domains
* technical leads evaluating NLP approaches and trade-offs
* researchers seeking a structured overview of the field

## Prerequisites
* `solid` programming experience in any modern language
* basic linear algebra (vectors, matrices, `dot` products)
* basic probability and statistics
* familiarity with `machine learning` fundamentals (training, loss, evaluation)
* exposure to `deep learning` concepts is helpful but not required

## Required Knowledge
* `Machine Learning` Fundamentals (or equivalent experience)
* Basic Linear Algebra and Probability (or equivalent experience)

## Objectives
* understand the linguistic and statistical foundations that `make` text processing distinctive
* apply tokenization, morphological analysis, and normalization to diverse languages and scripts
* build and evaluate language models from `n`-grams through transformers
* implement and reason about word and contextual embeddings
* design and train text classifiers, including sentiment and topic models
* perform sequence labeling for part-of-speech tagging and named entity recognition
* parse syntactic and dependency structures with both classical and neural methods
* apply attention and transformer architectures to standard NLP tasks
* approach machine translation, summarization, question answering, and dialogue
* deploy retrieval-augmented systems and evaluate their behavior
* select appropriate evaluation metrics and recognize their limitations
* navigate the practical concerns of building NLP systems at scale

## Outline
<!-- chapter: foundations-of-natural-language, duration: 2h -->
* Foundations of Natural Language
    * What makes language hard for computers
        * Ambiguity at every level (lexical, syntactic, semantic, pragmatic)
        * Productivity and the long tail of expressions
        * Context dependence and world knowledge
        * Variation across languages, dialects, and registers
    * Levels of linguistic analysis
        * Phonology, morphology, syntax, semantics, pragmatics, discourse
        * The role each level plays in computational systems
        * How modern NLP systems compress these levels
    * A brief history of NLP
        * Symbolic and rule-based era
        * Statistical revolution
        * `Deep learning` era
        * Pre-trained language model era
        * Large language model era
    * NLP tasks at a glance
        * Classification, tagging, parsing, generation, retrieval
        * Where each task fits in real-world applications
<!-- chapter: text-preprocessing-and-tokenization, duration: 3h -->
* Text Preprocessing and Tokenization
    * Character encoding and `Unicode`
        * `UTF-8` and the wider encoding landscape
        * Normalization forms (`NFC`, NFD, NFKC, `NFKD`)
        * Common pitfalls: invisible characters, mixed scripts, RTL
    * Sentence segmentation
        * Punctuation-based segmentation and its failures
        * Statistical segmentation
        * Cross-language considerations
    * Word tokenization
        * Whitespace and punctuation tokenization
        * Language-specific tokenization (`Chinese`, Japanese, `Arabic`)
        * Edge cases: contractions, hyphens, URLs, emoji
    * Subword tokenization
        * Byte-Pair Encoding (`BPE`)
        * `WordPiece` and `SentencePiece`
        * Unigram language model tokenization
        * Trade-offs of vocabulary size and granularity
    * Normalization techniques
        * Case folding and case sensitivity
        * Accent and diacritic handling
        * Stop words and when to remove them
        * Spelling correction as preprocessing
<!-- chapter: morphology-and-lemmatization, duration: 2h -->
* Morphology and Lemmatization
    * Morphological structure
        * Stems, roots, affixes, clitics
        * Inflectional vs derivational morphology
        * Productive vs unproductive morphological processes
    * Stemming
        * Porter stemmer
        * Snowball stemmers
        * Language-dependent stemming
        * Limitations and over-stemming
    * Lemmatization
        * Dictionary-based lemmatization
        * Lemmatization with morphological analyzers
        * `WordNet` and lexical resources
    * Morphologically rich languages
        * Agglutinative languages (`Finnish`, Turkish, `Hungarian`)
        * Templatic morphology (`Arabic`, `Hebrew`)
        * Why subword tokenization helps
    * Compounding and decomposition
        * `German` compound nouns
        * Decompounding strategies
        * `When` morphology matters less in modern systems
<!-- chapter: text-representation-classical, duration: 3h -->
* Classical Text Representation
    * Bag of words
        * Document-term matrix
        * Vocabulary construction and pruning
        * Sparse representation and storage
    * `n`-gram features
        * Unigrams, bigrams, trigrams
        * Skip-grams
        * Feature explosion and feature selection
    * Term weighting
        * Raw counts vs binary indicators
        * `TF`-`IDF` derivation and intuition
        * Variants: log normalization, sublinear weighting
        * `BM25` and probabilistic weighting
    * Latent semantic representations
        * Latent Semantic Analysis (`LSA`) and SVD
        * Latent Dirichlet Allocation (`LDA`) intuition
        * Topic modeling as dimensionality reduction
    * Distributional `hypothesis`
        * "You shall know a word by the company it keeps"
        * Co-occurrence matrices
        * Pointwise Mutual Information (`PMI`) and variants
        * From counts to vectors
<!-- chapter: word-embeddings, duration: 3h -->
* Word Embeddings
    * From counts to dense vectors
        * Why dense representations help
        * Properties: composition, similarity, analogy
    * `Word2Vec`
        * `CBOW` (Continuous Bag of Words)
        * Skip-gram with negative sampling
        * Hierarchical softmax
        * Hyperparameters and their effect
    * `GloVe`
        * Co-occurrence matrix factorization
        * Loss function derivation
        * `GloVe` vs `Word2Vec` comparison
    * `FastText`
        * Subword embeddings
        * Out-of-vocabulary handling
        * Cross-lingual extensions
    * Embedding properties and analysis
        * Word similarity benchmarks
        * Analogy tasks (`king` minus man plus woman = `queen`)
        * Visualization with `t-SNE` and `UMAP`
        * Bias in embeddings
    * Cross-lingual embeddings
        * Mapping monolingual spaces
        * `MUSE` and adversarial alignment
        * Multilingual joint training
<!-- chapter: language-models, duration: 3h -->
* Language Models
    * What a language model is
        * Probability over sequences
        * Next-token prediction
        * Perplexity and intrinsic evaluation
    * `n`-gram language models
        * Maximum likelihood estimation
        * Smoothing: Laplace, Good-Turing, Kneser-Ney
        * Back-off and interpolation
        * Limits of `n`-gram models
    * Neural language models
        * Feed-forward neural language models
        * Bengio et al. and the move to embeddings
        * Recurrent language models
    * Modern language models
        * From task-specific to pre-trained
        * Causal vs masked language modeling
        * The paradigm shift to pretrain-then-finetune
    * Evaluation
        * Perplexity
        * Cross-entropy and bits-per-character
        * Downstream-task evaluation
<!-- chapter: sequence-models-rnn-lstm-gru, duration: 3h -->
* Sequence Models: `RNN`, `LSTM`, GRU
    * Why sequences need special architectures
        * Variable length, order matters
        * Information flow over time
    * Recurrent Neural Networks
        * Mathematical formulation
        * Backpropagation through time (`BPTT`)
        * Vanishing and exploding gradients
    * Long Short-Term Memory (`LSTM`)
        * Gates: input, forget, output
        * Cell state and hidden state
        * Why `LSTM` mitigates vanishing gradients
    * Gated Recurrent Unit (`GRU`)
        * Reset and update gates
        * `GRU` vs `LSTM` empirically
    * Bidirectional models
        * `BiLSTM` and information from the future
        * Use cases and limitations
    * Encoder-decoder architectures
        * Sequence-to-sequence framing
        * Bottleneck problem
        * The setup for attention
<!-- chapter: attention-and-transformers, duration: 3h -->
* Attention and Transformers
    * Attention mechanism
        * From bottlenecks to soft alignment
        * Bahdanau and Luong attention
        * Scaled `dot`-product attention
    * Self-attention
        * Queries, keys, values
        * Why self-attention captures long-range dependencies
        * Computational complexity
    * Multi-head attention
        * Multiple representation subspaces
        * Implementation details
    * The transformer architecture
        * Encoder block and decoder block
        * Positional encoding (sinusoidal, learned, rotary)
        * Layer normalization placement
        * Residual connections
    * Pre-training objectives
        * Masked language modeling (`BERT`)
        * Causal language modeling (`GPT`)
        * Span corruption (`T5`)
        * Denoising and contrastive objectives
    * Scaling laws
        * Compute, data, parameter trade-offs
        * Emergent capabilities
<!-- chapter: pretrained-language-models, duration: 3h -->
* Pre-trained Language Models
    * Encoder models
        * `BERT` architecture and pre-training
        * `RoBERTa`, ALBERT, `DistilBERT`
        * `ELECTRA` and replaced-token detection
    * Decoder models
        * `GPT` family
        * Causal language modeling at scale
    * Encoder-decoder models
        * `T5` and text-to-text framing
        * `BART` and `mBART`
        * Multilingual variants
    * Tokenizers and pre-training data
        * `WordPiece`, BPE, `SentencePiece`
        * Pre-training corpora and filtering
        * Data quality and downstream effects
    * Fine-tuning
        * Adding task-specific heads
        * Learning rate schedules and warmup
        * Catastrophic forgetting
        * Parameter-efficient fine-tuning (`LoRA`, prefix tuning, adapters)
    * Few-shot and zero-shot learning
        * In-context learning
        * Prompt design as a fine-tuning alternative
        * Limits of zero-shot generalization
<!-- chapter: text-classification-and-sentiment, duration: 3h -->
* Text Classification and Sentiment Analysis
    * Classification fundamentals
        * Binary, multi-class, multi-label
        * Class imbalance and sampling strategies
        * Threshold selection and operating points
    * Classical classifiers
        * Naive Bayes
        * Logistic regression with `n`-gram features
        * Support Vector Machines for text
        * Feature engineering vs feature learning
    * Neural classifiers
        * CNN for text classification
        * `LSTM` and `BiLSTM` classifiers
        * Transformer-based classification
    * Sentiment analysis
        * Polarity, intensity, subjectivity
        * Aspect-based sentiment analysis
        * Emotion detection beyond polarity
        * Lexicon-based vs learned approaches
    * Domain adaptation
        * Out-of-domain generalization
        * Transfer learning strategies
        * Continued pre-training on domain text
    * Evaluation
        * Accuracy, precision, recall, `F1`
        * Confusion matrices for multi-class
        * `ROC` and precision-recall curves
        * Stratified cross-validation for class imbalance
<!-- chapter: sequence-labeling, duration: 2h -->
* Sequence Labeling
    * Sequence labeling as a unifying task
        * Per-token labels with structural constraints
        * `IOB`, BIOES, `BILOU` tagging schemes
    * Hidden Markov Models
        * Generative formulation
        * Viterbi decoding
        * Forward-backward and Baum-Welch
    * Maximum Entropy Markov Models
        * Discriminative formulation
        * Label bias problem
    * Conditional Random Fields
        * Linear-chain `CRF`
        * Why `CRF`s solve the label bias problem
        * Training and inference
    * Neural sequence labelers
        * `BiLSTM`-`CRF`
        * Transformer-based labelers
        * Span-based approaches
<!-- chapter: part-of-speech-tagging, duration: 2h -->
* Part-of-Speech Tagging
    * POS tagging concepts
        * Universal Dependencies tag set
        * Language-specific fine-grained tag sets
        * Tagging as sequence labeling
    * Classical POS taggers
        * Rule-based taggers
        * `HMM`-based taggers
        * Brill tagger and transformation-based learning
    * Neural POS taggers
        * Character-level features
        * Transformer-based taggers
        * Multi-task tagging
    * Evaluation and error analysis
        * Token accuracy and sentence accuracy
        * Tag confusion analysis
        * Out-of-vocabulary token handling
    * POS tagging across languages
        * Morphologically rich languages
        * Low-resource languages
<!-- chapter: named-entity-recognition, duration: 2h -->
* Named Entity Recognition
    * NER fundamentals
        * Standard entity types (`PER`, ORG, LOC, `MISC`)
        * Domain-specific entity types
        * Nested and overlapping entities
    * Classical NER
        * Rule-based and gazetteer-driven NER
        * `CRF`-based NER
        * Feature engineering for NER
    * Neural NER
        * `BiLSTM`-`CRF`
        * Transformer-based NER
        * Span-based and biaffine NER
    * Few-shot and zero-shot NER
        * Prompt-based extraction
        * Distantly supervised NER
        * Cross-lingual NER
    * Evaluation
        * Entity-level vs token-level metrics
        * Partial-match scoring
        * Error analysis: boundary errors vs type errors
<!-- chapter: syntactic-parsing, duration: 3h -->
* Syntactic Parsing
    * Constituency parsing
        * Phrase-structure grammars
        * Context-free grammars
        * `CKY` parsing algorithm
        * Probabilistic context-free grammars
    * Dependency parsing
        * Dependency relations and `UD` annotation
        * Projective vs non-projective dependencies
        * Transition-based parsing (arc-standard, arc-eager)
        * Graph-based parsing (`MST` and biaffine)
    * Neural parsing
        * Stack `LSTM` parsers
        * Biaffine attention parsers
        * Transformer-based parsing
    * Information from parses
        * Subject-verb-object extraction
        * Relation extraction with parse paths
        * Linguistic queries over parsed text
    * Evaluation
        * Unlabeled and labeled attachment scores
        * Bracketing scores for constituency
        * Cross-corpus consistency
<!-- chapter: semantics-and-meaning, duration: 2h -->
* Semantics and Meaning
    * Lexical semantics
        * Word senses and polysemy
        * Synonymy, antonymy, hypernymy, meronymy
        * `WordNet` and lexical knowledge bases
    * Word sense disambiguation
        * Lesk algorithm
        * Supervised disambiguation
        * Contextual embeddings as implicit disambiguation
    * Compositional semantics
        * From words to phrases to sentences
        * Combinators and tree composition
        * Why `deep learning` largely sidesteps formal composition
    * Semantic role labeling
        * Predicates and arguments
        * `PropBank` and `FrameNet`
        * Neural `SRL` systems
    * Coreference resolution
        * Anaphora and cataphora
        * Mention detection and clustering
        * Neural coreference systems
    * Textual entailment
        * Natural language inference
        * Entailment, contradiction, neutrality
        * Datasets and challenges
<!-- chapter: machine-translation, duration: 3h -->
* Machine Translation
    * Translation as a problem
        * Word-level translation and its limits
        * Phrase-based statistical translation
        * The translation adequacy-fluency trade-off
    * Statistical machine translation
        * `IBM` models and word alignment
        * Phrase-based decoding
        * Reordering models
    * Neural machine translation
        * Sequence-to-sequence with attention
        * Transformer-based translation
        * Subword tokenization for translation
    * Multilingual and zero-shot translation
        * Multilingual transformers
        * Pivot translation
        * Zero-shot directions
    * Evaluation
        * `BLEU` and its variants
        * `chrF`, METEOR, `TER`
        * `BERTScore` and learned metrics
        * Human evaluation: adequacy and fluency
    * Practical considerations
        * Domain adaptation
        * Terminology and glossaries
        * Low-resource translation
<!-- chapter: summarization, duration: 2h -->
* Summarization
    * Extractive summarization
        * Sentence scoring (`TF`-`IDF`, position, centrality)
        * Graph-based methods (`TextRank`, `LexRank`)
        * Selecting non-redundant sentences
    * Abstractive summarization
        * Sequence-to-sequence framing
        * Pointer-generator networks
        * Transformer-based summarization (`BART`, T5, `PEGASUS`)
    * Hybrid approaches
        * Extract-then-abstract
        * Constrained generation
    * Multi-document summarization
        * Aggregating across sources
        * Redundancy and contradiction handling
    * Evaluation
        * `ROUGE` family of metrics
        * `BERTScore` for summarization
        * Faithfulness and hallucination
        * Human evaluation criteria
<!-- chapter: question-answering, duration: 3h -->
* Question Answering
    * Question taxonomy
        * Factoid vs non-factoid
        * Open-domain vs closed-domain
        * Extractive vs abstractive answers
    * Reading comprehension
        * `SQuAD` and span extraction
        * Multi-hop reasoning datasets
        * Models for span extraction
    * Open-domain QA
        * Retrieval + reader pipelines
        * Dense passage retrieval
        * `RAG` and retrieval-augmented generation
    * Knowledge-grounded QA
        * Knowledge bases and structured KBs
        * `SPARQL` and semantic parsing
        * Hybrid retrieval over text and structure
    * Conversational QA
        * Multi-turn context
        * Reference resolution within conversation
    * Evaluation
        * Exact match and `F1`
        * Faithfulness to retrieved context
        * Adversarial robustness
<!-- chapter: dialogue-systems, duration: 2h -->
* Dialogue Systems
    * Task-oriented dialogue
        * Slot filling and intent classification
        * Dialogue state tracking
        * Policy and response generation
    * Open-domain dialogue
        * Retrieval-based vs generative responses
        * Persona and style consistency
        * Coherence over long conversations
    * Conversational `AI` components
        * Natural language understanding
        * Dialogue manager
        * Natural language generation
        * Speech integration
    * Modern `LLM`-based dialogue
        * In-context learning for dialogue
        * Tool use and function calling
        * Memory and conversation history
    * Evaluation
        * Task completion metrics
        * Engagement and quality metrics
        * Safety and toxicity
<!-- chapter: information-retrieval-and-rag, duration: 3h -->
* Information Retrieval and Retrieval-Augmented Generation
    * Classical retrieval
        * Boolean retrieval
        * Vector space model
        * `BM25` and probabilistic retrieval
        * Inverted indexes
    * Neural retrieval
        * Dense passage retrieval
        * Bi-encoders vs cross-encoders
        * Approximate nearest neighbor search (`HNSW`, FAISS, `ScaNN`)
    * Embedding models for retrieval
        * `Sentence-transformers`
        * Domain adaptation for retrieval
        * Hard negative mining
    * Retrieval-augmented generation
        * Architecture overview
        * Context window management
        * Citing sources and grounding
    * Hybrid retrieval
        * Combining lexical and dense signals
        * Reranking with cross-encoders
        * Query rewriting and expansion
    * Evaluation
        * Recall@K and precision@K
        * `MRR` and `nDCG`
        * Faithfulness in `RAG` outputs
<!-- chapter: text-generation-and-decoding, duration: 2h -->
* Text Generation and Decoding
    * Autoregressive generation
        * Token-by-token generation
        * Stop conditions
    * Decoding strategies
        * Greedy decoding
        * Beam search
        * Sampling: temperature, top-`k`, top-`p` (nucleus)
        * Contrastive decoding
    * Constrained generation
        * Hard constraints (regex, grammar)
        * Soft constraints (style, length)
        * Logit biasing
    * Controlled generation
        * Prompt engineering
        * Conditional generation with control codes
        * Reinforcement learning from human feedback (`RLHF`)
    * Evaluation
        * Perplexity (intrinsic)
        * Automated quality metrics
        * Human evaluation rubrics
        * Hallucination detection
<!-- chapter: large-language-models-in-context, duration: 2h -->
* Large Language Models in Context
    * What scale changed
        * Few-shot generalization
        * Instruction following
        * Chain-of-thought reasoning
    * Alignment
        * Supervised fine-tuning on instructions
        * Preference data and reward models
        * `RLHF` and `DPO`
        * Constitutional `AI` and self-critique
    * Tool use and agents
        * Function calling
        * Multi-step planning
        * Memory and state across turns
    * Limitations
        * Hallucination and confabulation
        * Reasoning failures
        * Context window constraints
        * Cost and latency
    * Evaluation in the `LLM` era
        * Capability benchmarks (`MMLU`, BBH, `HumanEval`)
        * Quality benchmarks (`MT-Bench`, `Arena`)
        * Domain-specific evaluation
<!-- chapter: multilingual-and-low-resource-nlp, duration: 2h -->
* Multilingual and Low-Resource NLP
    * The long tail of languages
        * Resource availability across the world's languages
        * Why English dominates the literature
    * Multilingual models
        * `mBERT`, `XLM-RoBERTa`, `mT5`
        * Cross-lingual transfer
        * Curse of multilinguality
    * Low-resource techniques
        * Cross-lingual transfer learning
        * Data augmentation through translation
        * Distant supervision
        * Active learning for annotation
    * Code-switching and mixed text
        * Detection and tokenization
        * Modeling considerations
    * Writing systems
        * Logographic, syllabic, alphabetic, abugida
        * Script normalization
        * Romanization and transliteration
<!-- chapter: bias-fairness-and-ethics, duration: 2h -->
* Bias, Fairness, and Ethics in NLP
    * Bias in language data
        * Sources of bias in corpora
        * Stereotypes encoded in embeddings
        * Demographic disparities in performance
    * Measuring bias
        * Word embedding bias tests (`WEAT`)
        * Disparity metrics for downstream tasks
        * Counterfactual evaluation
    * Mitigating bias
        * Debiasing embeddings
        * Data augmentation and balancing
        * Adversarial training
        * Limits of mitigation
    * Toxicity and content moderation
        * Detecting toxic language
        * Cross-demographic robustness
        * The labeling problem
    * Privacy
        * Memorization in language models
        * Differential privacy in NLP
        * `PII` detection and redaction
    * Misuse and dual-use concerns
        * Generated misinformation
        * Deepfake text
        * Responsible deployment
<!-- chapter: nlp-evaluation-methodology, duration: 2h -->
* NLP Evaluation Methodology
    * Choosing a metric
        * Task-specific metrics review
        * `When` metrics disagree
        * Human evaluation as ground truth
    * Statistical significance
        * Bootstrap and permutation tests
        * Multiple-comparisons corrections
        * Confidence intervals on benchmarks
    * Benchmark caveats
        * Train/test contamination
        * Benchmark saturation
        * Distribution shift
        * Constructing held-out evaluations
    * Error analysis
        * Slicing performance by category
        * Failure modes in production
        * Adversarial and stress testing
    * Reproducibility
        * Random seeds and variance
        * Reporting standards
        * Reproducing published results
<!-- chapter: practical-nlp-systems, duration: 2h -->
* Practical NLP Systems
    * From notebook to production
        * Packaging models
        * Inference servers and batching
        * Caching and request deduplication
    * Latency and throughput
        * Quantization (`INT8`, `INT4`, mixed precision)
        * Distillation and pruning
        * `ONNX` and accelerator-specific runtimes
        * `KV` cache for autoregressive generation
    * Cost considerations
        * Model size vs accuracy
        * Self-hosted vs `API`
        * Per-request economics
    * Monitoring NLP systems
        * Distribution shift detection
        * Quality regression
        * Hallucination and faithfulness monitoring
    * Data flywheels
        * Logging predictions for retraining
        * Active learning loops
        * Labeling infrastructure
    * Team workflows
        * Versioning data, models, and prompts
        * Experiment tracking
        * Model cards and documentation

## Copyright
Mark Veltzer [mark.veltzer@gmail.com](mailto:mark.veltzer@gmail.com), © 2026
