# `langsan` is a sanitization library for language models

![Build Status](https://github.com/mdegans/langsan/actions/workflows/tests.yaml/badge.svg)
[![codecov](https://codecov.io/gh/mdegans/langsan/branch/main/graph/badge.svg)](https://codecov.io/gh/mdegans/langsan)

Out of a desire to be first to market, [many companies from OpenAI to Anthropic](https://arstechnica.com/security/2024/10/ai-chatbots-can-read-and-write-invisible-text-creating-an-ideal-covert-channel/) are releasing language models without proper input or output sanitization. This can lead to a variety of safety and security issues, including but not limited to human-invisible adversarial attacks, data leakage, and generation of harmful content.

`langsan` provides immutable string wrappers guaranteeing their contents are within restricted unicode ranges, generally those only officially supported by a particular language model. All unicode code blocks are available as features. The library is designed to be fast and efficient, with minimal overhead.
