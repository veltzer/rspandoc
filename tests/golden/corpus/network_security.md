---
tags:
  - security:security
  - networking:networking
level: intermediate
category: networking
duration_hours: 24
audience:
  - audiences:security-professionals
  - audiences:sysadmins
  - audiences:devops
---

<!-- course: network_security -->
# Network Security

## Description
Network security is a critical discipline in today's interconnected world, where organizations face increasingly sophisticated threats targeting their network infrastructure. This course provides a comprehensive understanding of network security principles, technologies, and practices. Students will learn how to protect networks using firewalls, VPNs, encryption, and intrusion detection systems, as well as how to analyze traffic, respond to incidents, and design secure network architectures following modern zero trust principles.

## Duration
24 hours / 3 days

## Intended Audience
* Security professionals responsible for protecting network infrastructure
* System administrators managing firewalls, VPNs, and network devices
* Network engineers transitioning to security-focused roles
* IT professionals preparing for security certifications
* `DevOps` engineers implementing network security in cloud environments

## Prerequisites
* `Solid` understanding of `TCP`/`IP` networking fundamentals
* Familiarity with `Linux` and `Windows` command-line tools
* Basic understanding of network protocols (`HTTP`, `DNS`, `DHCP`)
* Experience with network device configuration (routers, switches)
* Understanding of basic cryptographic concepts is helpful

## Required Knowledge
* `Linux` Fundamentals (or equivalent experience)

## Objectives
* Understand `TCP`/`IP` security vulnerabilities and attack vectors
* Configure and manage firewalls including next-generation firewalls
* Deploy and manage `VPN` technologies (`IPSec`, `WireGuard`, `OpenVPN`)
* Implement `TLS`/`SSL` encryption and manage `PKI` infrastructure
* Secure `DNS` infrastructure using `DNSSEC`, `DoH`, and `DoT`
* Design and implement network segmentation and zero trust architectures
* Perform network traffic analysis using `Wireshark` and `tcpdump`
* Develop incident response procedures for network security events

## Outline
<!-- chapter: network-security-fundamentals, duration: 1h -->
* Network Security Fundamentals
    * Security principles: confidentiality, integrity, availability
    * Threat landscape and common attack vectors
    * Defense in depth strategy
    * Security models and frameworks (`NIST`, `ISO 27001`)
    * Risk assessment and management for network infrastructure
<!-- chapter: tcp-ip-security, duration: 1h -->
* `TCP`/`IP` Security
    * `TCP`/`IP` stack vulnerabilities
    * `ARP` spoofing and poisoning
    * `IP` spoofing and fragmentation attacks
    * `TCP` session hijacking and SYN flooding
    * `ICMP` attacks and reconnaissance
    * `UDP` flood attacks
    * Network scanning and enumeration techniques
<!-- chapter: firewalls, duration: 2h -->
* Firewalls
    * Stateless packet filtering
    * Stateful inspection firewalls
    * Application layer firewalls
    * Next-generation firewalls (NGFW) features and capabilities
    * `Firewall` rule design and management
    * `iptables` and `nftables` on `Linux`
    * Cloud-native firewalls and security groups
    * Web Application Firewalls (`WAF`)
<!-- chapter: intrusion-detection-and-prevention-systems, duration: 1h -->
* Intrusion Detection and Prevention Systems
    * `IDS` vs `IPS`: concepts and deployment modes
    * Signature-based detection
    * Anomaly-based detection
    * Network-based `IDS`/`IPS` (Snort, Suricata)
    * Host-based intrusion detection
    * Tuning and managing false positives
    * Integration with `SIEM` platforms
<!-- chapter: vpn-technologies, duration: 2h -->
* `VPN` Technologies
    * `VPN` concepts and use cases
    * `IPSec` architecture: AH, ESP, IKE
    * `IPSec` tunnel mode vs transport mode
    * `WireGuard`: architecture, configuration, and key management
    * `OpenVPN`: setup, certificates, and configuration
    * Site-to-site vs remote access VPNs
    * Split tunneling and full tunneling
    * `VPN` performance and troubleshooting
<!-- chapter: tls-ssl-deep-dive, duration: 2h -->
* `TLS`/`SSL` Deep Dive
    * `TLS` handshake process (```TLS`` 1.2` and `TLS` 1.3)
    * Certificate types and validation levels
    * Cipher suites and algorithm negotiation
    * Perfect forward secrecy
    * Common `TLS` vulnerabilities and mitigations
    * `TLS` inspection and interception considerations
    * Testing `TLS` configuration (ssllabs, testssl.`sh`)
<!-- chapter: pki-and-certificate-management, duration: 2h -->
* `PKI` and Certificate Management
    * Public Key Infrastructure components
    * Certificate Authorities: root, intermediate, and leaf certificates
    * Certificate lifecycle management
    * Certificate revocation: CRL and `OCSP`
    * `Let's Encrypt` and automated certificate management (`ACME`)
    * Certificate pinning
    * Private `CA` deployment and management
<!-- chapter: dns-security, duration: 2h -->
* `DNS` Security
    * `DNS` attack vectors: cache poisoning, amplification, tunneling
    * `DNSSEC`: signing, validation, and key management
    * `DNS` over `HTTPS` (`DoH`)
    * `DNS` over `TLS` (`DoT`)
    * `DNS` filtering and sinkholing
    * Protective `DNS` services
    * Monitoring `DNS` for security threats
<!-- chapter: network-segmentation, duration: 1h -->
* Network Segmentation
    * VLANs and network isolation
    * Micro-segmentation strategies
    * DMZ architecture and design
    * Network Access Control (NAC)
    * Software-defined segmentation
    * East-west vs north-south traffic control
<!-- chapter: zero-trust-architecture, duration: 2h -->
* Zero Trust Architecture
    * Zero trust principles: never trust, always verify
    * Identity-based access control
    * Device trust and posture assessment
    * Continuous verification and least privilege
    * Zero trust network access (`ZTNA`)
    * Implementing zero trust with existing infrastructure
    * `BeyondCorp` and similar frameworks
<!-- chapter: ddos-mitigation, duration: 2h -->
* `DDoS` Mitigation
    * `DDoS` attack types: volumetric, protocol, application layer
    * Detection and early warning
    * On-premise mitigation techniques
    * Cloud-based `DDoS` protection services
    * Rate limiting and traffic shaping
    * Anycast and scrubbing centers
    * Incident response for `DDoS` attacks
<!-- chapter: wireless-security, duration: 1h -->
* Wireless Security
    * Wireless security standards: WEP, WPA, `WPA2`, `WPA3`
    * `WPA3` features: SAE, enhanced open, management frame protection
    * Rogue access point detection
    * Evil twin attacks and mitigation
    * Enterprise wireless security (802.1X, `RADIUS`)
    * Wireless intrusion detection
<!-- chapter: network-monitoring-and-traffic-analysis, duration: 2h -->
* Network Monitoring and Traffic Analysis
    * Packet capture with `tcpdump`
    * Protocol analysis with `Wireshark`
    * `Wireshark` filters and analysis techniques
    * NetFlow and sFlow analysis
    * Network baseline and anomaly detection
    * Full packet capture vs metadata analysis
    * Encrypted traffic analysis techniques
<!-- chapter: siem-integration, duration: 1h -->
* `SIEM` Integration
    * `SIEM` concepts and architecture
    * Log collection from network devices
    * Correlation rules and use cases
    * Security event triage and investigation
    * Common `SIEM` platforms overview
<!-- chapter: incident-response, duration: 1h -->
* Incident Response
    * Network incident response procedures
    * Evidence collection and chain of custody
    * Network forensics techniques
    * Containment and eradication strategies
    * Recovery and lessons learned
    * Communication during security incidents
<!-- chapter: compliance, duration: 1h -->
* Compliance
    * `PCI`-DSS network security requirements
    * Network segmentation for compliance
    * Logging and audit trail requirements
    * Vulnerability scanning and penetration testing requirements
    * Documentation and policy requirements

## Copyright
Mark Veltzer [mark.veltzer@gmail.com](mailto:mark.veltzer@gmail.com), © 2026
