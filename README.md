
# Table of Contents

-   [Demos](#orgeb24b56)
-   [Installation](#org0eea0a2)
    -   [Prerequisite](#org0d7f7b5)
    -   [Option 1: Cargo install](#org79fae5e)
    -   [Option 2: Prebuilt binaries](#org9144220)
    -   [Option 3: Build from source](#org679dee5)
-   [About](#org01f0d28)
    -   [How Mafa works](#org411b9b5)
    -   [Why Mafa](#org2d730eb)
    -   [Background](#orgfb4c250)
-   [Mafa Components](#org03bc856)
    -   [More and more](#org7a5d690)
-   [Contributing](#orgb70ab12)
-   [License](#org1a944b9)



<a id="orgeb24b56"></a>

# Demos

*(captured by [peek](https://github.com/phw/peek))*

<table>
    <tr>
    <td><img src="demo-twtl.gif" height="160px"/><p align="center"><a href="<https://twitter.com>">Twitter Timeline</a></p></td>
    <td><img src="demo-gtrans.gif" height="160px"/><p align="center"><a href="<https://translate.google.com>">Google Translate</a></p></td>
    <td><img src="demo-camd.gif" height="160px"/><p align="center"><a href="<https://dictionary.cambridge.org/us/dictionary/english>">Cambridge Dictionary</a></p></td>
    </tr>
</table>


<a id="org0eea0a2"></a>

# Installation


<a id="org0d7f7b5"></a>

## Prerequisite

Mafa does not work alone, below are programs that it
depends on:

1.  Firefox
    -   version: 91 or later
    -   edition: ESR or Latest.


<a id="org79fae5e"></a>

## Option 1: Cargo install

If you have Cargo installed already, then you can

    cargo install mafa
    
    # check installed version
    mafa --version


<a id="org9144220"></a>

## Option 2: Prebuilt binaries

Check [releases](https://github.com/micl2e2/mafa/releases).


<a id="org679dee5"></a>

## Option 3: Build from source

WIP


<a id="org01f0d28"></a>

# About

Mafa is an in-terminal web browser companion. It resides in terminal,
help perple browse websites' content readily and efficiently.
Mafa accomplishs its tasks by [Mafa Components](#org7bfeaed).

Mafa develops for the ones who want to benefit from Web's openness
as much as possible.

However, Mafa is **NOT** suitable for the following tasks: 

-   Browse websites without a web browser.  (Mafa needs Firefox)

-   Capture every detail of a website.  (Open your favorite web browser
    directly)

-   Crawl a whole website and extract all its data.  (A dedicated web
    crawler does a better job)


<a id="org411b9b5"></a>

## How Mafa works

Mafa leverages [WebDriver](https://www.w3.org/TR/webdriver) to achieve its goals. More specifically,
Mozilla's [GeckoDriver](https://github.com/mozilla/geckodriver) is in use. With WebDriver, Mafa can act like a
human, browsing websites naturally for its user.


<a id="org2d730eb"></a>

## Why Mafa


### Mafa is Usable & Convenient

Unlike other counterparts, Mafa strives to balance usability and
convenience: Mafa will try its best to finish the task on its own or
instruct users to open web browser directly if it cannot
perfectly handle the situation(such as in cases where the website
is equipped with CAPTCHA or requires user login). What Mafa tries to
be is a browser companion, **not** a replacement.


### Mafa is Neutral

The underlying WebDriver backs by a nearly full-functional web
browser. Overall, Mafa default **not** to subjectively strip any feature
a website user or provider can take advantage of, just like on a
normal full-functional web browser.

Therefore there is no reason for providers to particularly prevent
Mafa from accessing their websites, which likely leads to a negative
result for **both** sides.

It is noteworthy that Mafa does not wipe out the user identity by
default, as a regular web browser does. It is essential for website
providers because while many websites abuse user privacy, there are
always ones collecting it for a good reason, such as [Ecosia](https://www.ecosia.org).


### Mafa is Stable & Long-lasting

One of Mafa's goals is to handle websites stably for a relatively
****long**** period. Modern web pages are famous for their dynamic
characteristic. However, Mafa can handle those dynamic and
unpredictable web pages as effortlessly as the static ones. 


<a id="orgfb4c250"></a>

## Background

Although Mafa is initially developed for (**M**)aking (**A**)PI (**F**)ree
(**A**)gain, it is not realistic. Instead of freeing APIs, Mafa
frees the text-form data behind the APIs. Here "free" is the same word
defined in [What is Free Software?](https://www.gnu.org/philosophy/free-sw.en.html), i.e., as in "free speech", not as
in "free beer".

Some websites provide their data *publicly* but do not 
publish corresponding APIs to access it, while others offer their data
*publicly* in their carefully designed websites and APIs but with even
more carefully designed pricing. Those websites are blocking users
from accessing their *public* data by either not providing APIs or
providing ones with non-trivial barriers, examples of disrespecting
users' freedom.

Mafa is the one who commits to protexting web users' freedom. It
tries its best to achieve the initial goal: as long as the data is
publicly accessible to all users without discrimination, the APIs to
access it should be as well. 


<a id="org03bc856"></a>

# Mafa Components

<a id="org7bfeaed"></a>

*Mafa Components* are child programs that rely on [WebDriver](https://www.w3.org/TR/webdriver) to do 
their job. Each module has a fixed, predefined website url and has a 
specific task for that site. With components, users can browse websites
without interacting with web browsers directly.

Note that Mafa supports wbsites *selectively* rather than *arbitrarily*,
the selected ones are:

-   TWTL: Query Twitter users' timeline.

-   GTRANS: Query translation from Google Translate.

-   CAMD: Query word definition from Cambridge Dictionary.

-   IMODE: Interactive mode.


<a id="org7a5d690"></a>

## More and more

Mafa is open in its heart! If your favorite websites are not
listed here, you can [submit a features request](https://github.com/micl2e2/mafa/issues/new) or write a component
for your favorite website, as long as that site meets the following
requirements:

1.  Not shut down in the foreseeable future.
2.  The valuable data is in text form.
3.  The functionality of public-offered APIs is limited.


<a id="orgb70ab12"></a>

# Contributing

Mafa is still in early development, any contribution is welcomed!


<a id="org1a944b9"></a>

# License

Mafa is proudly licensed under GPLv3.

See LICENSE-GPL for details.

