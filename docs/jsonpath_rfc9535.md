# RFC 9535: JSONPath: Query Expressions for JSON

**Internet Engineering Task Force (IETF)**  
**Request for Comments: 9535**  
**Category: Standards Track**  
**ISSN: 2070-1721**  

**Editors:**
- S. Gössner, Ed. (Fachhochschule Dortmund)
- G. Normington, Ed.
- C. Bormann, Ed. (Universität Bremen TZI)

**Date:** February 2024

## Abstract

JSONPath defines a string syntax for selecting and extracting JSON (RFC 8259) values from within a given JSON value.

## Status of This Memo

This is an Internet Standards Track document.

This document is a product of the Internet Engineering Task Force (IETF). It represents the consensus of the IETF community. It has received public review and has been approved for publication by the Internet Engineering Steering Group (IESG). Further information on Internet Standards is available in Section 2 of RFC 7841.

Information about the current status of this document, any errata, and how to provide feedback on it may be obtained at https://www.rfc-editor.org/info/rfc9535.

## Copyright Notice

Copyright (c) 2024 IETF Trust and the persons identified as the document authors. All rights reserved.

This document is subject to BCP 78 and the IETF Trust's Legal Provisions Relating to IETF Documents (https://trustee.ietf.org/license-info) in effect on the date of publication of this document. Please review these documents carefully, as they describe your rights and restrictions with respect to this document. Code Components extracted from this document must include Revised BSD License text as described in Section 4.e of the Trust Legal Provisions and are provided without warranty as described in the Revised BSD License.

## Table of Contents

1. [Introduction](#1-introduction)
   - 1.1. [Terminology](#11-terminology)
     - 1.1.1. [JSON Values as Trees of Nodes](#111-json-values-as-trees-of-nodes)
   - 1.2. [History](#12-history)
   - 1.3. [JSON Values](#13-json-values)
   - 1.4. [Overview of JSONPath Expressions](#14-overview-of-jsonpath-expressions)
     - 1.4.1. [Identifiers](#141-identifiers)
     - 1.4.2. [Segments](#142-segments)
     - 1.4.3. [Selectors](#143-selectors)
     - 1.4.4. [Summary](#144-summary)
   - 1.5. [JSONPath Examples](#15-jsonpath-examples)
2. [JSONPath Syntax and Semantics](#2-jsonpath-syntax-and-semantics)
   - 2.1. [Overview](#21-overview)
     - 2.1.1. [Syntax](#211-syntax)
     - 2.1.2. [Semantics](#212-semantics)
     - 2.1.3. [Example](#213-example)
   - 2.2. [Root Identifier](#22-root-identifier)
     - 2.2.1. [Syntax](#221-syntax)
     - 2.2.2. [Semantics](#222-semantics)
     - 2.2.3. [Examples](#223-examples)
   - 2.3. [Selectors](#23-selectors)
     - 2.3.1. [Name Selector](#231-name-selector)
       - 2.3.1.1. [Syntax](#2311-syntax)
       - 2.3.1.2. [Semantics](#2312-semantics)
       - 2.3.1.3. [Examples](#2313-examples)
     - 2.3.2. [Wildcard Selector](#232-wildcard-selector)
       - 2.3.2.1. [Syntax](#2321-syntax)
       - 2.3.2.2. [Semantics](#2322-semantics)
       - 2.3.2.3. [Examples](#2323-examples)
     - 2.3.3. [Index Selector](#233-index-selector)
       - 2.3.3.1. [Syntax](#2331-syntax)
       - 2.3.3.2. [Semantics](#2332-semantics)
       - 2.3.3.3. [Examples](#2333-examples)
     - 2.3.4. [Array Slice Selector](#234-array-slice-selector)
       - 2.3.4.1. [Syntax](#2341-syntax)
       - 2.3.4.2. [Semantics](#2342-semantics)
       - 2.3.4.3. [Examples](#2343-examples)
     - 2.3.5. [Filter Selector](#235-filter-selector)
       - 2.3.5.1. [Syntax](#2351-syntax)
       - 2.3.5.2. [Semantics](#2352-semantics)
       - 2.3.5.3. [Examples](#2353-examples)
   - 2.4. [Function Extensions](#24-function-extensions)
     - 2.4.1. [Type System for Function Expressions](#241-type-system-for-function-expressions)
     - 2.4.2. [Type Conversion](#242-type-conversion)
     - 2.4.3. [Well-Typedness of Function Expressions](#243-well-typedness-of-function-expressions)
     - 2.4.4. [length() Function Extension](#244-length-function-extension)
     - 2.4.5. [count() Function Extension](#245-count-function-extension)
     - 2.4.6. [match() Function Extension](#246-match-function-extension)
     - 2.4.7. [search() Function Extension](#247-search-function-extension)
     - 2.4.8. [value() Function Extension](#248-value-function-extension)
     - 2.4.9. [Examples](#249-examples)
   - 2.5. [Segments](#25-segments)
     - 2.5.1. [Child Segment](#251-child-segment)
       - 2.5.1.1. [Syntax](#2511-syntax)
       - 2.5.1.2. [Semantics](#2512-semantics)
       - 2.5.1.3. [Examples](#2513-examples)
     - 2.5.2. [Descendant Segment](#252-descendant-segment)
       - 2.5.2.1. [Syntax](#2521-syntax)
       - 2.5.2.2. [Semantics](#2522-semantics)
       - 2.5.2.3. [Examples](#2523-examples)
   - 2.6. [Semantics of null](#26-semantics-of-null)
     - 2.6.1. [Examples](#261-examples)
   - 2.7. [Normalized Paths](#27-normalized-paths)
     - 2.7.1. [Examples](#271-examples)
3. [IANA Considerations](#3-iana-considerations)
   - 3.1. [Registration of Media Type application/jsonpath](#31-registration-of-media-type-applicationjsonpath)
   - 3.2. [Function Extensions Subregistry](#32-function-extensions-subregistry)
4. [Security Considerations](#4-security-considerations)
   - 4.1. [Attack Vectors on JSONPath Implementations](#41-attack-vectors-on-jsonpath-implementations)
   - 4.2. [Attack Vectors on How JSONPath Queries Are Formed](#42-attack-vectors-on-how-jsonpath-queries-are-formed)
   - 4.3. [Attacks on Security Mechanisms That Employ JSONPath](#43-attacks-on-security-mechanisms-that-employ-jsonpath)
5. [References](#5-references)
   - 5.1. [Normative References](#51-normative-references)
   - 5.2. [Informative References](#52-informative-references)
- [Appendix A: Collected ABNF Grammars](#appendix-a-collected-abnf-grammars)
- [Appendix B: Inspired by XPath](#appendix-b-inspired-by-xpath)
  - B.1. [JSONPath and XPath](#b1-jsonpath-and-xpath)
- [Appendix C: JSON Pointer](#appendix-c-json-pointer)

## 1. Introduction

JSON [RFC8259] is a popular representation format for structured data values. JSONPath defines a string syntax for selecting and extracting JSON values from within a given JSON value.

In relation to JSON Pointer [RFC6901], JSONPath is not intended as a replacement but as a more powerful companion. See Appendix C.

### 1.1. Terminology

The key words "MUST", "MUST NOT", "REQUIRED", "SHALL", "SHALL NOT", "SHOULD", "SHOULD NOT", "RECOMMENDED", "NOT RECOMMENDED", "MAY", and "OPTIONAL" in this document are to be interpreted as described in BCP 14 [RFC2119] [RFC8174] when, and only when, they appear in all capitals, as shown here.

The grammatical rules in this document are to be interpreted as ABNF, as described in [RFC5234]. ABNF terminal values in this document define Unicode scalar values rather than their UTF-8 encoding. For example, the Unicode PLACE OF INTEREST SIGN (U+2318) would be defined in ABNF as %x2318.

Functions are referred to using the function name followed by a pair of parentheses, as in fname().

The terminology of [RFC8259] applies except where clarified below. The terms "primitive" and "structured" are used to group different kinds of values as in Section 1 of [RFC8259]. JSON objects and arrays are structured; all other values are primitive. Definitions for "object", "array", "number", and "string" remain unchanged. Importantly, "object" and "array" in particular do not take on a generic meaning, such as they would in a general programming context.

The terminology of [RFC9485] applies.

Additional terms used in this document are defined below.

**Value:** As per [RFC8259], a data item conforming to the generic data model of JSON, i.e., primitive data (numbers, text strings, and the special values null, true, and false), or structured data (JSON objects and arrays). [RFC8259] focuses on the textual representation of JSON values and does not fully define the value abstraction assumed here.

**Member:** A name/value pair in an object. (A member is not itself a value.)

**Name:** The name (a string) in a name/value pair constituting a member. This is also used in [RFC8259], but that specification does not formally define it. It is included here for completeness.

**Element:** A value in a JSON array.

**Index:** An integer that identifies a specific element in an array.

**Query:** Short name for a JSONPath expression.

**Query Argument:** Short name for the value a JSONPath expression is applied to.

**Location:** The position of a value within the query argument. This can be thought of as a sequence of names and indexes navigating to the value through the objects and arrays in the query argument, with the empty sequence indicating the query argument itself. A location can be represented as a Normalized Path (defined below).

**Node:** The pair of a value along with its location within the query argument.

**Root Node:** The unique node whose value is the entire query argument.

**Root Node Identifier:** The expression $, which refers to the root node of the query argument.

**Current Node Identifier:** The expression @, which refers to the current node in the context of the evaluation of a filter expression (described later).

**Children (of a node):** If the node is an array, the nodes of its elements; if the node is an object, the nodes of its member values. If the node is neither an array nor an object, it has no children.

**Descendants (of a node):** The children of the node, together with the children of its children, and so forth recursively. More formally, the "descendants" relation between nodes is the transitive closure of the "children" relation.

**Depth (of a descendant node within a value):** The number of ancestors of the node within the value. The root node of the value has depth zero, the children of the root node have depth one, their children have depth two, and so forth.

**Nodelist:** A list of nodes. While a nodelist can be represented in JSON, e.g., as an array, this document does not require or assume any particular representation.

**Parameter:** Formal parameter (of a function) that can take a function argument (an actual parameter) in a function expression.

**Normalized Path:** A form of JSONPath expression that identifies a node in a value by providing a query that results in exactly that node. Each node in a query argument is identified by exactly one Normalized Path (we say that the Normalized Path is "unique" for that node), and to be a Normalized Path for a specific query argument, the Normalized Path needs to identify exactly one node. This is similar to, but syntactically different from, a JSON Pointer [RFC6901]. Note: This definition is based on the syntactical definition in Section 2.7; JSONPath expressions that identify a node in a value but do not conform to that syntax are not Normalized Paths.

**Unicode Scalar Value:** Any Unicode [UNICODE] code point except high-surrogate and low-surrogate code points (in other words, integers in the inclusive base 16 ranges, either 0 to D7FF or E000 to 10FFFF). JSONPath queries are sequences of Unicode scalar values.

**Segment:** One of the constructs that selects children ([<selectors>]) or descendants (..[<selectors>]) of an input value.

**Selector:** A single item within a segment that takes the input value and produces a nodelist consisting of child nodes of the input value.

**Singular Query:** A JSONPath expression built from segments that have been syntactically restricted in a certain way (Section 2.3.5.1) so that, regardless of the input value, the expression produces a nodelist containing at most one node. Note: JSONPath expressions that always produce a singular nodelist but do not conform to the syntax in Section 2.3.5.1 are not singular queries.

#### 1.1.1. JSON Values as Trees of Nodes

This document models the query argument as a tree of JSON values, each with its own node. A node is either the root node or one of its descendants.

This document models the result of applying a query to the query argument as a nodelist (a list of nodes).

Nodes are the selectable parts of the query argument. The only parts of an object that can be selected by a query are the member values. Member names and members (name/value pairs) cannot be selected. Thus, member values have nodes, but members and member names do not. Similarly, member values are children of an object, but members and member names are not.

### 1.2. History

This document is based on Stefan Gössner's popular JSONPath proposal (dated 2007-02-21) [JSONPath-orig], builds on the experience from the widespread deployment of its implementations, and provides a normative specification for it.

Appendix B describes how JSONPath was inspired by XML's XPath [XPath].

JSONPath was intended as a lightweight companion to JSON implementations in programming languages such as PHP and JavaScript, so instead of defining its own expression language, like XPath did, JSONPath delegated parts of a query to the underlying runtime, e.g., JavaScript's eval() function. As JSONPath was implemented in more environments, JSONPath expressions became decreasingly portable. For example, regular expression processing was often delegated to a convenient regular expression engine.

This document aims to remove such implementation-specific dependencies and serve as a common JSONPath specification that can be used across programming languages and environments. This means that backwards compatibility is not always achieved; a design principle of this document is to go with a "consensus" between implementations even if it is rough, as long as that does not jeopardize the objective of obtaining a usable, stable JSON query language.

The term _JSONPath_ was chosen because of the XPath inspiration and also because the outcome of a query consists of _paths_ identifying nodes in the JSON query argument.

### 1.3. JSON Values

The JSON value a JSONPath query is applied to is, by definition, a valid JSON value. A JSON value is often constructed by parsing a JSON text.

The parsing of a JSON text into a JSON value and what happens if a JSON text does not represent valid JSON are not defined by this document. Sections 4 and 8 of [RFC8259] identify specific situations that may conform to the grammar for JSON texts but are not interoperable uses of JSON, as they may cause unpredictable behavior. This document does not attempt to define predictable behavior for JSONPath queries in these situations.

Specifically, the "Semantics" subsections of Sections 2.3.1, 2.3.2, 2.3.5, and 2.5.2 describe behavior that becomes unpredictable when the JSON value for one of the objects under consideration was constructed out of JSON text that exhibits multiple members for a single object that share the same member name ("duplicate names"; see Section 4 of [RFC8259]). Also, when selecting a child by name (Section 2.3.1) and comparing strings (Section 2.3.5.2.2), it is assumed these strings are sequences of Unicode scalar values; the behavior becomes unpredictable if they are not (Section 8.2 of [RFC8259]).

### 1.4. Overview of JSONPath Expressions

A JSONPath expression is applied to a JSON value, known as the query argument. The output is a nodelist.

A JSONPath expression consists of an identifier followed by a series of zero or more segments, each of which contains one or more selectors.

#### 1.4.1. Identifiers

The root node identifier $ refers to the root node of the query argument, i.e., to the argument as a whole.

The current node identifier @ refers to the current node in the context of the evaluation of a filter expression (Section 2.3.5).

#### 1.4.2. Segments

Segments select children ([<selectors>]) or descendants (..[<selectors>]) of an input value.

Segments can use _bracket notation_, for example:

$['store']['book'][0]['title']

or the more compact _dot notation_, for example:

$.store.book[0].title

Bracket notation contains one or more (comma-separated) selectors of any kind. Selectors are detailed in the next section.

A JSONPath expression may use a combination of bracket and dot notations.

This document treats the bracket notations as canonical and defines the shorthand dot notation in terms of bracket notation. Examples and descriptions use shorthand where convenient.

#### 1.4.3. Selectors

A name selector, e.g., 'name', selects a named child of an object.

An index selector, e.g., 3, selects an indexed child of an array.

In the expression [*], a wildcard * (Section 2.3.2) selects all children of a node, and in the expression ..[*], it selects all descendants of a node.

An array slice start:end:step (Section 2.3.4) selects a series of elements from an array, giving a start position, an end position, and an optional step value that moves the position from the start to the end.

A filter expression ?<logical-expr> selects certain children of an object or array, as in:

$.store.book[?@.price < 10].title

#### 1.4.4. Summary

Table 1 provides a brief overview of JSONPath syntax.

| Syntax Element   | Description                                    |
|==================|================================================|
| $                | root node identifier (Section 2.2)             |
| @                | current node identifier (Section 2.3.5)        |
|                  | (valid only within filter selectors)           |
| [<selectors>]    | child segment (Section 2.5.1): selects         |
|                  | zero or more children of a node                |
| .name            | shorthand for ['name']                         |
| .*               | shorthand for [*]                              |
| ..[<selectors>]  | descendant segment (Section 2.5.2):            |
|                  | selects zero or more descendants of a node     |
| ..name           | shorthand for ..['name']                       |
| ..*              | shorthand for ..[*]                            |
| 'name'           | name selector (Section 2.3.1): selects a       |
|                  | named child of an object                       |
| *                | wildcard selector (Section 2.3.2): selects     |
|                  | all children of a node                         |
| 3                | index selector (Section 2.3.3): selects an     |
|                  | indexed child of an array (from 0)             |
| 0:100:5          | array slice selector (Section 2.3.4):          |
|                  | start:end:step for arrays                      |
| ?<logical-expr>  | filter selector (Section 2.3.5): selects       |
|                  | particular children using a logical            |
|                  | expression                                     |
| length(@.foo)    | function extension (Section 2.4): invokes      |
|                  | a function in a filter expression              |

**Table 1: Overview of JSONPath Syntax**

### 1.5. JSONPath Examples

This section is informative. It provides examples of JSONPath expressions.

The examples are based on the simple JSON value shown in Figure 1, representing a bookstore (which also has a bicycle).

#### Figure 1: Example JSON Value

```json
{
  "store": {
    "book": [
      {
        "category": "reference",
        "author": "Nigel Rees",
        "title": "Sayings of the Century",
        "price": 8.95
      },
      {
        "category": "fiction",
        "author": "Evelyn Waugh",
        "title": "Sword of Honour",
        "price": 12.99
      },
      {
        "category": "fiction",
        "author": "Herman Melville",
        "title": "Moby Dick",
        "isbn": "0-553-21311-3",
        "price": 8.99
      },
      {
        "category": "fiction",
        "author": "J. R. R. Tolkien",
        "title": "The Lord of the Rings",
        "isbn": "0-395-19395-8",
        "price": 22.99
      }
    ],
    "bicycle": {
      "color": "red",
      "price": 399
    }
  }
}
```

#### Table 2: Example JSONPath Expressions and Their Intended Results

| JSONPath | Intended Result |
|----------|----------------|
| `$.store.book[*].author` | the authors of all books in the store |
| `$..author` | all authors |
| `$.store.*` | all things in the store, which are some books and a red bicycle |
| `$.store..price` | the prices of everything in the store |
| `$..book[2]` | the third book |
| `$..book[2].author` | the third book's author |
| `$..book[2].publisher` | empty result: the third book does not have a "publisher" member |
| `$..book[-1]` | the last book in order |
| `$..book[0,1]` | the first two books |
| `$..book[:2]` | the first two books |
| `$..book[?@.isbn]` | all books with an ISBN number |
| `$..book[?@.price<10]` | all books cheaper than 10 |
| `$..*` | all member values and array elements contained in the input value |

## 2. JSONPath Syntax and Semantics

### 2.1. Overview

A JSONPath _expression_ is a string that, when applied to a JSON value (the _query argument_), selects zero or more nodes of the argument and outputs these nodes as a nodelist.

A query MUST be encoded using UTF-8. The grammar for queries given in this document assumes that its UTF-8 form is first decoded into Unicode scalar values as described in [RFC3629]; implementation approaches that lead to an equivalent result are possible.

A string to be used as a JSONPath query needs to be _well-formed_ and _valid_. A string is a well-formed JSONPath query if it conforms to the ABNF syntax in this document. A well-formed JSONPath query is valid if it also fulfills both semantic requirements posed by this document, which are as follows:

1. Integer numbers in the JSONPath query that are relevant to the JSONPath processing (e.g., index values and steps) MUST be within the range of exact integer values defined in Internet JSON (I-JSON) (see Section 2.2 of [RFC7493]), namely within the interval [-(2^53)+1, (2^53)-1].

2. Uses of function extensions MUST be _well-typed_, as described in Section 2.4.3.

A JSONPath implementation MUST raise an error for any query that is not well-formed and valid. The well-formedness and the validity of JSONPath queries are independent of the JSON value the query is applied to. No further errors relating to the well-formedness and the validity of a JSONPath query can be raised during application of the query to a value. This clearly separates well-formedness/validity errors in the query from mismatches that may actually stem from flaws in the data.

#### 2.1.1. Syntax

Syntactically, a JSONPath query consists of a root identifier ($), which stands for a nodelist that contains the root node of the query argument, followed by a possibly empty sequence of _segments_.

```abnf
jsonpath-query      = root-identifier segments
segments            = *(S segment)

B                   = %x20 /    ; Space
                      %x09 /    ; Horizontal tab
                      %x0A /    ; Line feed or New line
                      %x0D      ; Carriage return
S                   = *B        ; optional blank space
```

#### 2.1.2. Semantics

In this document, the semantics of a JSONPath query define the required results and do not prescribe the internal workings of an implementation. This document may describe semantics in a procedural step-by-step fashion; however, such descriptions are normative only in the sense that any implementation MUST produce an identical result but not in the sense that implementers are required to use the same algorithms.

The semantics are that a valid query is executed against a value (the _query argument_) and produces a nodelist (i.e., a list of zero or more nodes of the value).

The query is a root identifier followed by a sequence of zero or more segments, each of which is applied to the result of the previous root identifier or segment and provides input to the next segment. These results and inputs take the form of nodelists.

The nodelist resulting from the root identifier contains a single node (the query argument). The nodelist resulting from the last segment is presented as the result of the query. Depending on the specific API, it might be presented as an array of the JSON values at the nodes, an array of Normalized Paths referencing the nodes, or both -- or some other representation as desired by the implementation. Note: An empty nodelist is a valid query result.

A segment operates on each of the nodes in its input nodelist in turn, and the resultant nodelists are concatenated in the order of the input nodelist they were derived from to produce the result of the segment. A node may be selected more than once and appears that number of times in the nodelist. Duplicate nodes are not removed.

A syntactically valid segment MUST NOT produce errors when executing the query. This means that some operations that might be considered erroneous, such as using an index lying outside the range of an array, simply result in fewer nodes being selected.

As a consequence of this approach, if any of the segments produces an empty nodelist, then the whole query produces an empty nodelist.

If the semantics of a query give an implementation a choice of producing multiple possible orderings, a particular implementation may produce distinct orderings in successive runs of the query.

#### 2.1.3. Example

Consider this example. With the query argument `{"a":[{"b":0},{"b":1},{"c":2}]}`, the query `$.a[*].b` selects the following list of nodes (denoted here by their values): 0, 1.

The query consists of $ followed by three segments: .a, [*], and .b.

First, $ produces a nodelist consisting of just the query argument.

Next, .a selects from any object input node and selects the node of any member value of the input node corresponding to the member name "a". The result is again a list containing a single node: `[{"b":0},{"b":1},{"c":2}]`.

Next, [*] selects all the elements from the input array node. The result is a list of three nodes: `{"b":0}`, `{"b":1}`, and `{"c":2}`.

Finally, .b selects from any object input node with a member name b and selects the node of the member value of the input node corresponding to that name. The result is a list containing 0, 1. This is the concatenation of three lists: two of length one containing 0, 1, respectively, and one of length zero.

### 2.2. Root Identifier

#### 2.2.1. Syntax

Every JSONPath query (except those inside filter expressions; see Section 2.3.5) MUST begin with the root identifier $.

```abnf
root-identifier     = "$"
```

#### 2.2.2. Semantics

The root identifier $ represents the root node of the query argument.

#### 2.2.3. Examples

In each of the following examples, the query argument is `{"k": "v"}`:

| Query | Result | Result Paths | Comment |
|-------|--------|--------------|---------|
| `$` | `{"k": "v"}` | `$` | root node |

### 2.3. Selectors

Selectors appear only inside bracketed selection expressions. The expression `[<selectors>]` selects a subset of the children of the input value.

There are various kinds of selectors that can appear inside brackets:

- name: select the child with the given name
- index: select the child with the given index  
- array slice: select a contiguous slice of children
- wildcard: select all children
- filter: select children matching a logical expression

Multiple selectors can be combined within a single pair of brackets as a union. For example, `[0, 2]` selects children 0 and 2 of the input value.

Selectors that might select a single node MUST NOT be multi-valued. For example, a single name selector can select at most one child of an object, and a single index selector can select at most one element of an array.

When a selector could result in selection of the same node more than once, the node appears in the nodelist for each time it matches the selector.

#### 2.3.1. Name Selector

##### 2.3.1.1. Syntax

A name selector `'<name>'` matches an object's member whose name equals `<name>`.

```abnf
name-selector       = string-literal
string-literal      = %x22 *double-quoted %x22 /     ; "string"
                      %x27 *single-quoted %x27        ; 'string'

double-quoted       = unescaped /
                      %x5C (                        ; \
                          %x22 /                    ; "    quotation mark  U+0022
                          %x5C /                    ; \    reverse solidus U+005C  
                          %x2F /                    ; /    solidus         U+002F
                          %x62 /                    ; b    backspace       U+0008
                          %x66 /                    ; f    form feed       U+000C
                          %x6E /                    ; n    line feed       U+000A
                          %x72 /                    ; r    carriage return U+000D
                          %x74 /                    ; t    tab             U+0009
                          (%x75 4HEXDIG) )          ; uXXXX                U+XXXX

single-quoted       = unescaped /
                      %x5C (                        ; \
                          %x27 /                    ; '    apostrophe      U+0027
                          %x5C /                    ; \    reverse solidus U+005C
                          %x2F /                    ; /    solidus         U+002F
                          %x62 /                    ; b    backspace       U+0008
                          %x66 /                    ; f    form feed       U+000C
                          %x6E /                    ; n    line feed       U+000A
                          %x72 /                    ; r    carriage return U+000D
                          %x74 /                    ; t    tab             U+0009
                          (%x75 4HEXDIG) )          ; uXXXX                U+XXXX

unescaped           = %x20-21 /                     ; see RFC 8259
                      %x23-26 /                     ; omit "
                      %x28-5B /                     ; omit '
                      %x5D-10FFFF                   ; omit \
```

##### 2.3.1.2. Semantics

A name selector produces a nodelist consisting of at most one node, the child member value of an object whose name equals the string value of the name selector.

If the input value is not an object, the name selector produces an empty nodelist. If the input value is an object but has no member whose name equals the string value of the name selector, the name selector produces an empty nodelist.

##### 2.3.1.3. Examples

| Query | Result | Result Paths | Comment |
|-------|--------|--------------|---------|
| `$['a']` | `"b"` | `$['a']` | object member |
| `$['d']` | | | no object member |

### 2.3.2. Wildcard Selector

#### 2.3.2.1. Syntax

The wildcard selector consists of an asterisk:

```abnf
wildcard-selector   = "*"
```

#### 2.3.2.2. Semantics

A wildcard selector produces a nodelist consisting of all the children of the input value.

- If the input value is an object, the wildcard selector produces a nodelist consisting of the member values of the object.
- If the input value is an array, the wildcard selector produces a nodelist consisting of the elements of the array.
- If the input value is neither an object nor an array, the wildcard selector produces an empty nodelist.

#### 2.3.2.3. Examples

| Query | Result | Result Paths | Comment |
|-------|--------|--------------|---------|
| `$[*]` | `"a"`, `"b"` | `$[0]`, `$[1]` | array elements |
| `$.store.book[*].author` | `"Nigel Rees"`, `"Evelyn Waugh"`, `"Herman Melville"`, `"J. R. R. Tolkien"` | `$.store.book[0].author`, `$.store.book[1].author`, `$.store.book[2].author`, `$.store.book[3].author` | authors of all books |

### 2.3.3. Index Selector

#### 2.3.3.1. Syntax

An index selector `<index>` matches the element of an array at the specified index.

```abnf
index-selector      = int                           ; decimal integer

int                 = "0" /                         ; zero
                      (["-"] 1*9 *DIGIT)            ; nonzero
```

Negative indices count from the end of the array, with -1 referring to the last element, -2 to the penultimate element, and so forth.

#### 2.3.3.2. Semantics

An index selector produces a nodelist consisting of at most one element, the child element of an array at the given index.

If the input value is not an array, the index selector produces an empty nodelist. If the input value is an array but the index is out of range, the index selector produces an empty nodelist.

For a non-negative index i, `array[i]` refers to the element at index i. For a negative index i, `array[i]` refers to the element at index `len(array) + i` (where `len(array)` is the length of the array).

#### 2.3.3.3. Examples

| Query | Result | Result Paths | Comment |
|-------|--------|--------------|---------|
| `$[1]` | `"b"` | `$[1]` | array index |
| `$[-2]` | `"a"` | `$[0]` | negative array index |

### 2.3.4. Array Slice Selector

#### 2.3.4.1. Syntax

An array slice selector has the syntax `start:end:step`, reminiscent of Python array slicing.

```abnf
array-slice         = [start S] ":" S [end S] [":" S [step S]]

start               = int                           ; included in selection
end                 = int                           ; not included in selection  
step                = int                           ; default: 1
```

Each of `start`, `end`, and `step` is optional:

- If `start` is omitted, it defaults to 0 if `step >= 0` or to `len-1` if `step < 0`.
- If `end` is omitted, it defaults to `len` if `step >= 0` or to `-len-1` if `step < 0`.
- If `step` is omitted, it defaults to 1.
- `step` MUST NOT be 0.

#### 2.3.4.2. Semantics

An array slice selector produces a nodelist consisting of elements of an array. The slice selects a contiguous range of elements, but if `step != 1`, it selects only every |step|th element in that range.

#### 2.3.4.3. Examples

| Query | Result | Result Paths | Comment |
|-------|--------|--------------|---------|
| `$[1:3]` | `"b"`, `"c"` | `$[1]`, `$[2]` | slice with start:end |
| `$[:2]` | `"a"`, `"b"` | `$[0]`, `$[1]` | slice with end |
| `$[::2]` | `"a"`, `"c"` | `$[0]`, `$[2]` | slice with step |

### 2.3.5. Filter Selector

#### 2.3.5.1. Syntax

A filter selector has the syntax `?<logical-expr>`, where `<logical-expr>` is a logical expression.

```abnf
filter-selector     = "?" S logical-expr

logical-expr        = logical-or-expr
logical-or-expr     = logical-and-expr *(S "||" S logical-and-expr)
logical-and-expr    = basic-expr *(S "&&" S basic-expr)

basic-expr          = paren-expr /
                      comparison-expr /
                      test-expr

paren-expr          = [logical-not-op S] "(" S logical-expr S ")"
logical-not-op      = "!"

comparison-expr     = comparable S comparison-op S comparable
comparison-op       = "==" / "!=" / "<=" / ">=" / "<" / ">"
comparable          = literal /
                      singular-query /        ; See Section 2.3.5.1 for restrictions  
                      function-expr

test-expr           = [logical-not-op S] (singular-query / function-expr)
```

#### 2.3.5.2. Semantics

A filter selector produces a nodelist consisting of those children of the input value for which the filter expression is true.

#### 2.3.5.3. Examples

| Query | Result | Result Paths | Comment |
|-------|--------|--------------|---------|
| `$[?@.price < 10]` | books with price < 10 | | filter by price |
| `$[?@.isbn]` | books with ISBN | | filter by existence |

### 2.4. Function Extensions

This section defines the syntax and semantics for function extensions in JSONPath expressions.

#### 2.4.1. Type System for Function Expressions

Function expressions are typed. The type system includes these types:

- **ValueType**: The type of any JSON value.
- **LogicalType**: The type of the result of a test or logical expression (true or false).
- **NodesType**: The type of a nodelist.

#### 2.4.2. Type Conversion

Function arguments undergo type conversion according to these rules:

- A value of **ValueType** can be converted to **LogicalType** using the test expression conversion.
- A value of **NodesType** can be converted to **ValueType** if the nodelist has exactly one node.

#### 2.4.3. Well-Typedness of Function Expressions

A function expression is _well-typed_ if:

1. The function is known (i.e., is defined in this document or in a registered function extension).
2. The function is applied to the correct number of arguments.
3. All function arguments are well-typed.
4. All function arguments can be converted to the declared parameter types of the function.

#### 2.4.4. length() Function Extension

```abnf
length-function-expr = "length" "(" S function-expr S ")"
```

The `length()` function takes a **ValueType** argument and returns a number representing:

- For arrays: the number of elements
- For objects: the number of members  
- For strings: the number of Unicode scalar values
- For other JSON values: 1

#### 2.4.5. count() Function Extension

```abnf
count-function-expr = "count" "(" S function-expr S ")"
```

The `count()` function takes a **NodesType** argument and returns the number of nodes in the nodelist.

#### 2.4.6. match() Function Extension

```abnf
match-function-expr = "match" "(" S function-expr S "," S function-expr S ")"
```

The `match()` function takes two **ValueType** arguments: a string and a regular expression pattern. It returns **LogicalType** indicating whether the string matches the pattern.

#### 2.4.7. search() Function Extension

```abnf
search-function-expr = "search" "(" S function-expr S "," S function-expr S ")"
```

The `search()` function takes two **ValueType** arguments: a string and a regular expression pattern. It returns **LogicalType** indicating whether the pattern can be found anywhere in the string.

#### 2.4.8. value() Function Extension

```abnf
value-function-expr = "value" "(" S function-expr S ")"
```

The `value()` function takes a **NodesType** argument. If the nodelist contains exactly one node, it returns the **ValueType** value of that node. Otherwise, it produces nothing.

#### 2.4.9. Examples

| Function | Example | Description |
|----------|---------|-------------|
| `length()` | `length(@.authors)` | Number of authors |
| `count()` | `count($..book[*])` | Number of books |
| `match()` | `match(@.author, ".*Tolkien.*")` | Author name contains "Tolkien" |
| `search()` | `search(@.title, "Lord")` | Title contains "Lord" |
| `value()` | `value(@.price)` | Price value |

### 2.5. Segments

#### 2.5.1. Child Segment

##### 2.5.1.1. Syntax

A child segment consists of square brackets that contain one or more selectors:

```abnf
child-segment       = "[" S selector *(S "," S selector) S "]"

selector            = name-selector /
                      index-selector /
                      array-slice /
                      wildcard-selector /
                      filter-selector
```

##### 2.5.1.2. Semantics

A child segment produces a nodelist consisting of zero or more children of the input value. The child segment applies each selector to the input value, and the resultant nodelists are concatenated in the order of the selectors to produce the result of the child segment.

##### 2.5.1.3. Examples

| Query | Result | Comment |
|-------|--------|---------|
| `$['store']` | store object | single name selector |
| `$[0, 1]` | first two elements | multiple selectors |

#### 2.5.2. Descendant Segment

##### 2.5.2.1. Syntax

A descendant segment consists of two dots followed by a child segment:

```abnf
descendant-segment  = ".." S child-segment
```

##### 2.5.2.2. Semantics

A descendant segment produces a nodelist consisting of zero or more descendants of the input value. The descendant segment applies the child segment to every node at every depth in the input value.

##### 2.5.2.3. Examples

| Query | Result | Comment |
|-------|--------|---------|
| `$..price` | all prices | descendant search |
| `$..*` | all values | all descendants |

### 2.6. Semantics of null

The JSON `null` value is distinct from missing values. A query may select a node whose value is `null`, and a missing member is different from a member with a `null` value.

#### 2.6.1. Examples

| Query | JSON | Result | Comment |
|-------|------|--------|---------|
| `$.a` | `{"a": null}` | `null` | null value selected |
| `$.a` | `{}` | | missing member |

### 2.7. Normalized Paths

A _Normalized Path_ is a JSONPath expression that uniquely identifies a single node in a JSON value.

Normalized Paths have a normalized syntax:

- Use bracket notation exclusively
- Use single quotes for member names
- Use decimal integers for array indices (no leading zeros except for 0 itself)
- No whitespace except where required for parsing

#### 2.7.1. Examples

| Node | Normalized Path |
|------|----------------|
| Root | `$` |
| Member 'a' | `$['a']` |
| Array element 0 | `$[0]` |
| Nested | `$['store']['book'][0]['title']` |

## 3. IANA Considerations

### 3.1. Registration of Media Type application/jsonpath

IANA has registered the following media type:

- **Type name:** application
- **Subtype name:** jsonpath
- **Required parameters:** None
- **Optional parameters:** None
- **Encoding considerations:** JSONPath expressions are UTF-8 encoded Unicode text
- **Security considerations:** See Section 4
- **Interoperability considerations:** None
- **Published specification:** RFC 9535
- **Applications that use this media type:** Applications that process JSONPath expressions
- **Fragment identifier considerations:** None
- **Person & email address to contact for further information:** See Authors' Addresses
- **Intended usage:** COMMON
- **Restrictions on usage:** None
- **Author:** See Authors' Addresses
- **Change controller:** IETF

### 3.2. Function Extensions Subregistry

IANA has created a "JSONPath Function Extensions" subregistry within the "JavaScript Object Notation (JSON)" registry.

## 4. Security Considerations

### 4.1. Attack Vectors on JSONPath Implementations

JSONPath implementations need to be robust against various attack vectors:

1. **Complexity attacks**: Expressions that cause excessive computation or memory usage
2. **Parser attacks**: Malformed expressions that could exploit parser vulnerabilities  
3. **Regular expression attacks**: ReDoS (Regular Expression Denial of Service)

### 4.2. Attack Vectors on How JSONPath Queries Are Formed

Applications that construct JSONPath queries dynamically should:

1. Validate and sanitize user inputs
2. Use parameterized queries where possible
3. Implement appropriate access controls

### 4.3. Attacks on Security Mechanisms That Employ JSONPath

JSONPath should not be relied upon as a security mechanism by itself. Applications should implement proper authorization and validation independent of JSONPath filtering.

## 5. References

### 5.1. Normative References

- **[RFC2119]** Bradner, S., "Key words for use in RFCs to Indicate Requirement Levels", BCP 14, RFC 2119, DOI 10.17487/RFC2119, March 1997
- **[RFC3629]** Yergeau, F., "UTF-8, a transformation format of ISO 10646", STD 63, RFC 3629, DOI 10.17487/RFC3629, November 2003
- **[RFC5234]** Crocker, D., Ed., and P. Overell, "Augmented BNF for Syntax Specifications: ABNF", STD 68, RFC 5234, DOI 10.17487/RFC5234, January 2008
- **[RFC7493]** Bray, T., Ed., "The I-JSON Message Format", RFC 7493, DOI 10.17487/RFC7493, March 2015
- **[RFC8174]** Leiba, B., "Ambiguity of Uppercase vs Lowercase in RFC 2119 Key Words", BCP 14, RFC 8174, DOI 10.17487/RFC8174, May 2017
- **[RFC8259]** Bray, T., Ed., "The JavaScript Object Notation (JSON) Data Interchange Format", STD 90, RFC 8259, DOI 10.17487/RFC8259, December 2017
- **[RFC9485]** Bormann, C., "I-Regexp: An Interoperable Regular Expression Format", RFC 9485, DOI 10.17487/RFC9485, October 2023
- **[UNICODE]** The Unicode Consortium, "The Unicode Standard"

### 5.2. Informative References

- **[JSONPath-orig]** Gössner, S., "JSONPath - XPath for JSON", February 2007
- **[RFC6901]** Bryan, P., Ed., Zyp, K., and M. Nottingham, Ed., "JavaScript Object Notation (JSON) Pointer", RFC 6901, DOI 10.17487/RFC6901, April 2013
- **[XPath]** Clark, J. and S. DeRose, "XML Path Language (XPath) Version 1.0", W3C Recommendation, November 1999

## Appendix A. Collected ABNF Grammars

This appendix collects the ABNF grammar rules defined throughout this document:

```abnf
; JSONPath query
jsonpath-query      = root-identifier segments
root-identifier     = "$"
segments            = *(S segment)

; Segments  
segment             = child-segment / descendant-segment
child-segment       = "[" S selector *(S "," S selector) S "]"
descendant-segment  = ".." S child-segment

; Selectors
selector            = name-selector /
                      index-selector /
                      array-slice /
                      wildcard-selector /
                      filter-selector

; Name selector
name-selector       = string-literal
string-literal      = %x22 *double-quoted %x22 /     ; "string"
                      %x27 *single-quoted %x27        ; 'string'

; Index selector
index-selector      = int
int                 = "0" / (["-"] 1*9 *DIGIT)

; Array slice
array-slice         = [start S] ":" S [end S] [":" S [step S]]
start               = int
end                 = int  
step                = int

; Wildcard selector
wildcard-selector   = "*"

; Filter selector
filter-selector     = "?" S logical-expr
logical-expr        = logical-or-expr
logical-or-expr     = logical-and-expr *(S "||" S logical-and-expr)
logical-and-expr    = basic-expr *(S "&&" S basic-expr)

; Function expressions
function-expr       = length-function-expr /
                      count-function-expr /
                      match-function-expr /
                      search-function-expr /
                      value-function-expr

; Whitespace
B                   = %x20 / %x09 / %x0A / %x0D
S                   = *B
```

## Appendix B. Inspired by XPath

### B.1. JSONPath and XPath

JSONPath expressions apply to JSON values in the same way as XPath expressions are used in combination with an XML document. JSONPath uses `$` to refer to the root node of the query argument, similar to XPath's `/` at the front.

#### Table 21: Example XPath Expressions and Their JSONPath Equivalents

| XPath | JSONPath | Result |
|-------|----------|--------|
| `/store/book/author` | `$.store.book[*].author` | the authors of all books in the store |
| `//author` | `$..author` | all authors |
| `/store/*` | `$.store.*` | all things in store, which are some books and a red bicycle |
| `/store//price` | `$.store..price` | the prices of everything in the store |
| `//book[3]` | `$..book[2]` | the third book |
| `//book[last()]` | `$..book[-1]` | the last book in order |
| `//book[position()<3]` | `$..book[0,1]` or `$..book[:2]` | the first two books |
| `//book[isbn]` | `$..book[?@.isbn]` | filter all books with an ISBN number |
| `//book[price<10]` | `$..book[?@.price<10]` | filter all books cheaper than 10 |
| `//*` | `$..*` | all elements in an XML document; all member values and array elements contained in input value |

XPath has a lot more functionality (location paths in unabbreviated syntax, operators, and functions) than listed in this comparison. Moreover, there are significant differences in how the subscript operator works in XPath and JSONPath:

- Square brackets in XPath expressions always operate on the _node set_ resulting from the previous path fragment. Indices always start at 1.
- With JSONPath, square brackets operate on each of the nodes in the _nodelist_ resulting from the previous query segment. Array indices always start at 0.

## Appendix C. JSON Pointer

JSON Pointer [RFC6901] defines a string syntax for identifying a single value within a JSON document. JSONPath generalizes this by:

1. Allowing identification of multiple values
2. Supporting more complex selection logic (filters, slices, etc.)
3. Using a different syntax

A JSON Pointer can be converted to a JSONPath Normalized Path with these transformations:

- Replace `/` with `$['` and `']`
- Escape single quotes in member names
- Convert array indices to bracket notation

---

*This document represents the complete RFC 9535 specification.*