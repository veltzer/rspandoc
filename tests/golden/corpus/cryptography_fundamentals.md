---
tags:
  - security:security
  - concepts:algorithms
level: intermediate
category: security
duration_hours: 24
audience:
  - audiences:developers
  - audiences:security-professionals
  - audiences:devops
---

<!-- course: cryptography_fundamentals -->
# Cryptography Fundamentals

## Description
Cryptography is the foundation of modern information security. This course provides a comprehensive introduction to cryptographic concepts, algorithms, and protocols. Topics include symmetric and asymmetric encryption, hash functions, digital signatures, key exchange, `TLS`/`SSL`, `PKI`, password storage, and post-quantum cryptography. Participants will gain both theoretical understanding and practical experience using libraries such as `OpenSSL`, libsodium, and NaCl.

## Duration
24 hours / 3 days

## Intended Audience
* Software developers implementing security features
* Security engineers and analysts
* Backend developers working with authentication and encryption
* `DevOps` engineers managing certificates and secrets
* Anyone who needs to understand cryptography for their technical role

## Prerequisites
* Proficiency in at least one programming language (`Python`, C, `Java`, or similar)
* Basic understanding of computer science fundamentals (binary, data structures)
* Familiarity with networking concepts (`TCP`/`IP`, `HTTP`/`HTTPS`)
* Basic knowledge of security concepts (confidentiality, integrity, authentication)
* Comfort with command-line tools

## Required Knowledge
* `Python` Programming (or equivalent experience)

## Objectives
* Understand core cryptographic concepts and terminology
* Apply symmetric encryption algorithms (`AES`, ChaCha20) with appropriate modes
* Understand and use asymmetric encryption (`RSA`, ECC, Diffie-Hellman)
* Use hash functions (`SHA`-256, `SHA`-3, BLAKE) and MACs for integrity verification
* Implement and verify digital signatures
* Understand the `TLS`/`SSL` protocol and certificate management
* Apply proper password storage techniques (bcrypt, scrypt, Argon2)
* Recognize and avoid common cryptographic mistakes
* Use cryptographic libraries (`OpenSSL`, libsodium, NaCl) correctly
* Understand the landscape of post-quantum cryptography

## Outline
<!-- chapter: cryptography-concepts, duration: 1h -->
* Cryptography Concepts
    * History and evolution of cryptography
    * Security goals: confidentiality, integrity, authentication, non-repudiation
    * Kerckhoffs' principle and why secrecy of the algorithm is not security
    * Computational hardness and security parameters
    * Cryptographic primitives overview
    * Standards bodies and cryptographic recommendations (`NIST`, IETF)
<!-- chapter: symmetric-encryption, duration: 1h -->
* Symmetric Encryption
    * Symmetric encryption fundamentals and key management
    * `AES` (Advanced Encryption Standard) algorithm and key sizes
    * ChaCha20 stream cipher and its advantages
    * `When` to use `AES` vs ChaCha20
    * Key generation and secure key storage
<!-- chapter: block-cipher-modes, duration: 1h -->
* Block Cipher Modes
    * ECB (Electronic Codebook) and why it is insecure
    * CBC (Cipher Block Chaining) mode and initialization vectors
    * CTR (Counter) mode and parallelization
    * GCM (Galois/Counter Mode) for authenticated encryption
    * Choosing the right mode for your use case
    * Padding schemes and padding `oracle` attacks
<!-- chapter: asymmetric-encryption, duration: 2h -->
* Asymmetric Encryption
    * Public key cryptography fundamentals
    * `RSA` algorithm, key generation, and key sizes
    * Elliptic Curve Cryptography (ECC) and curve selection
    * Diffie-Hellman key exchange protocol
    * Elliptic Curve Diffie-Hellman (`ECDH`)
    * Hybrid encryption (combining symmetric and asymmetric)
    * Performance comparison of asymmetric algorithms
<!-- chapter: hash-functions, duration: 1h -->
* Hash Functions
    * Cryptographic hash function properties (collision resistance, preimage resistance)
    * `SHA`-256 and the `SHA`-2 family
    * `SHA`-3 (Keccak) and its design
    * BLAKE2 and BLAKE3 for high-performance hashing
    * Hash function use cases (integrity, fingerprinting, commitments)
    * Birthday attacks and hash length considerations
<!-- chapter: macs-and-hmac, duration: 1h -->
* MACs and `HMAC`
    * Message Authentication Codes concept
    * `HMAC` construction and security properties
    * CMAC and GMAC alternatives
    * Authenticated encryption with associated data (AEAD)
    * `AES`-GCM and ChaCha20-Poly1305 as AEAD constructions
    * `When` to use MACs vs digital signatures
<!-- chapter: digital-signatures, duration: 1h -->
* Digital Signatures
    * Digital signature concepts and use cases
    * `RSA` signatures (`PKCS#1 v1.5`, PSS)
    * `ECDSA` (Elliptic Curve Digital Signature Algorithm)
    * EdDSA and Ed25519 signatures
    * Signature verification and trust models
    * Code signing and document signing applications
<!-- chapter: key-exchange-protocols, duration: 1h -->
* Key Exchange Protocols
    * Key exchange problem and man-in-the-middle attacks
    * Diffie-Hellman key exchange in detail
    * Authenticated key exchange protocols
    * Forward secrecy and ephemeral keys
    * Key derivation functions (`HKDF`, `PBKDF2`)
    * Key agreement vs key transport
<!-- chapter: tls-ssl-protocol, duration: 2h -->
* `TLS`/`SSL` Protocol
    * `TLS` protocol overview and version history
    * `TLS 1.3` handshake and improvements over `TLS 1.2`
    * Cipher suite negotiation and selection
    * Certificate validation during handshake
    * Session resumption and 0-RTT
    * Common `TLS` misconfigurations and vulnerabilities
    * Testing `TLS` configuration
<!-- chapter: pki-and-certificates, duration: 2h -->
* `PKI` and Certificates
    * Public Key Infrastructure concepts
    * `X.509` certificate structure and fields
    * Certificate Authorities (`CA`) and trust chains
    * Certificate signing requests (CSR)
    * Certificate revocation (CRL, `OCSP`)
    * `Let's Encrypt` and automated certificate management
    * Certificate pinning and its trade-offs
<!-- chapter: key-management, duration: 2h -->
* Key Management
    * Key lifecycle (generation, distribution, storage, rotation, destruction)
    * Hardware Security Modules (`HSM`)
    * Key Management Services (`cloud KMS`)
    * Key wrapping and envelope encryption
    * Secret sharing schemes
    * Key escrow and recovery
<!-- chapter: password-storage, duration: 2h -->
* Password Storage
    * Why simple hashing is insufficient
    * Salting and its importance
    * bcrypt algorithm and cost factor tuning
    * scrypt and memory-hard functions
    * Argon2 (Argon2id) and `OWASP` recommendations
    * Password hashing benchmarks and parameter selection
    * Credential stuffing defense
<!-- chapter: random-number-generation, duration: 1h -->
* Random Number Generation
    * Importance of randomness in cryptography
    * Pseudorandom number generators (PRNG) vs cryptographically secure PRNGs (CSPRNG)
    * Operating system entropy sources (/dev/urandom, CryptGenRandom)
    * Common mistakes with random number generation
    * Testing randomness quality
<!-- chapter: post-quantum-cryptography, duration: 2h -->
* Post-Quantum Cryptography
    * Quantum computing threat to current cryptography
    * `NIST` post-quantum standardization process
    * Lattice-based cryptography (CRYSTALS-Kyber, CRYSTALS-Dilithium)
    * Hash-based signatures (SPHINCS+)
    * Migration planning and crypto agility
    * Hybrid classical/post-quantum approaches
<!-- chapter: common-cryptographic-mistakes, duration: 2h -->
* Common Cryptographic Mistakes
    * Rolling your own crypto
    * Using ECB mode for encryption
    * Reusing nonces and initialization vectors
    * Insufficient key lengths and weak algorithms
    * Improper random number generation
    * Timing attacks and side-channel vulnerabilities
    * Not using authenticated encryption
    * Hardcoded keys and secrets
<!-- chapter: cryptographic-libraries, duration: 2h -->
* Cryptographic Libraries
    * `OpenSSL` command-line and library usage
    * libsodium high-level `API` and best practices
    * NaCl (Networking and Cryptography Library) design philosophy
    * Language-specific libraries (`Python` cryptography, `Java` JCA/JCE, Go crypto)
    * Choosing the right library for your project
    * Keeping libraries updated and monitoring advisories

## Copyright
Mark Veltzer [mark.veltzer@gmail.com](mailto:mark.veltzer@gmail.com), © 2026
