# Security policy

## Reporting a vulnerability

**Do not open a public GitHub Issue, Discussion, or pull request to report a security vulnerability.** Public disclosure before a fix is available puts all Lore users at risk.

Report vulnerabilities through Epic Games' security channels:

**Primary — Epic Games HackerOne program:**
[https://hackerone.com/epicgames](https://hackerone.com/epicgames)

HackerOne is Epic's primary intake channel for external security reports. Submissions are triaged by a dedicated team and routed to the Lore maintainers. This channel supports private, structured disclosure and is eligible for the bug bounty program (see [below](#bug-bounty)).

**Alternative — email:**
[security@epicgames.com](mailto:security@epicgames.com)

Use this address if you prefer email over HackerOne, or if a HackerOne submission is not appropriate for your situation.

---

## Safe harbor

Epic doesn't pursue legal action against researchers acting in good faith under this policy.

---

## What to include in your report

Include as much of the following as you can:

- A description of the vulnerability and how it can be exploited
- Step-by-step reproduction instructions, or a minimal reproduction program
- The Lore version(s) affected (run `lore version` to check)
- Your operating system and CPU architecture
- An assessment of the impact — what can an attacker do if this is exploited?
- Your name and affiliation (for convenience - anonymity is also fine; just say so)
- Your preferred embargo duration, if you have one

---

## Response timeline

| Milestone | Target |
| --- | --- |
| Initial acknowledgement | Within 7 business days of receipt |
| Vulnerability assessment and severity determination | Within 30 business days |
| Patch development, testing, and release | 30–90 business days |
| Public disclosure | 90 business days (sooner for low-severity issues or active exploitation) |

If you do not receive an acknowledgement within 7 business days, follow up at [security@epicgames.com](mailto:security@epicgames.com) with "Lore security" in the subject line.

We may shorten this timeline if we learn the vulnerability is being actively exploited.

---

## Disclosure policy

We use three tracks based on severity and impact:

**Low-severity or limited-scope issues** — fixed in the normal release cycle, no embargo. The patch release notes credit the reporter.

**Moderate and high-severity issues** — 90-business-day coordinated disclosure with embargo. We publish details alongside the fixed release.

**Critical issues or active exploitation** — immediate dedicated security release. There is no advance embargo window.

---

## Scope

The following are in scope for Lore security reports:

- The `lore` CLI client
- `loreserver`
- The Lore wire protocol and on-disk data format
- Authentication and authorization mechanisms
- Language bindings (`lore-capi`, JavaScript, C#, Python, Go)

The following are generally out of scope:

- Vulnerabilities in third-party dependencies — report these to the upstream project; we coordinate patches but credit goes upstream
- Issues already publicly disclosed in GitHub Issues
- Denial-of-service attacks that require direct privileged access to the server

---

## Supported versions

Security patches are provided for the two most recent stable releases. Older releases do not receive security backports.

---

## Bug bounty

Epic maintains a public bug bounty program on [HackerOne](https://hackerone.com/epicgames). Lore is included in the program scope. Bounty amounts are defined on the HackerOne program page and depend on severity and impact.
