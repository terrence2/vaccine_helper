# (Adult) Vaccine Helper
---
Vaccine helper is there to give adults (with no a competent vaccine clinic in their area) a simple
way to track and schedule immunizations with all the incredible new vaccines that are available
since we were children.

The source for this tool is available on [GitHub](https://github.com/jimmycuadra/vaccine_helper).
[![dependency status](https://deps.rs/repo/github/terrence2/vaccine_helper/status.svg)](https://deps.rs/repo/github/terrence2/vaccine_helper)
[![Build Status](https://github.com/terrence2/vaccine_helper/workflows/CI/badge.svg)](https://github.com/terrence2/vaccine_helper/actions?workflow=CI)

## Warning
Usage of this (extremely simple) tool does not constitute medical advice. Please consult a doctor
and/or pharmacist before putting _anything_ in your body.

All data that the tool creates is stored locally, either in files on your disk in the native version
or in your browser's local storage for the web version. There are no ads, tracking cookies, or
backend database.

TODO: allow easy export/import of the save file on web.

## General Safety Information
There is overwhelming evidence that vaccines are both safe and effective. Vaccines are so effective
that we can't study the long-term effectiveness because there is nobody sick left to study.

The odds of a severe or worse vaccine reaction are, like shark attacks and lightning strikes,
low enough that computing accurate odds is impossible. Many vaccines are safe to the limits of
our ability to detect, with no reported severe reactions on record.

The WHO tracks adverse reaction rates for most vaccines. Check their safety sheets for any vaccines
you are concerned about specifically.
[WHO Vaccine Safety Sheets](https://www.who.int/teams/regulation-prequalification/regulation-and-safety/pharmacovigilance/guidance/reaction-rates-information-sheets)

# Getting started

## The website
The tool can be found at https://terrence2.github.io/vaccine_helper

## How to use this tool
The top section tracks immunizations. The middle section allows you to prioritize vaccines. The bottom
is a live-updating schedule that takes into account your past immunizations and future priorities.

## Profiles
You can track schedules for multiple people in the same save file using "Profiles". Create new profiles
and switch between them using the `Profiles` window. You can open this window in the menubar under
`File->Profiles...`.

# Hacking on this tool
See the instructions in the [instructions](/terrence2/instructions) file.

If you are using `nix`, you can pull in a full dev environment using the shell.nix file by running
`nix-shell` in your terminal. If using a different operating system, review the packages in shell.nix
and install the equivalents for your own system.

### Web Locally

You can compile your app to [WASM](https://en.wikipedia.org/wiki/WebAssembly) and publish it as a web page.

We use [Trunk](https://trunkrs.dev/) to build for web target.
1. Install the required target with `rustup target add wasm32-unknown-unknown`.
2. Install Trunk with `cargo install --locked trunk`.
3. Run `trunk serve` to build and serve on `http://127.0.0.1:8080`. Trunk will rebuild automatically if you edit the
   project.
4. Open `http://127.0.0.1:8080/index.html#dev` in a browser. See the warning below.

> `assets/sw.js` script will try to cache our app, and loads the cached version when it cannot connect to server
> allowing your app to work offline (like PWA).
> appending `#dev` to `index.html` will skip this caching, allowing us to load the latest builds during development.

### Web Deploy
This tool is hosted on github pages and auto-deploys when merging to the main branch.
