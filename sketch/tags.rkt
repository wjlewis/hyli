#lang racket

(provide html-tag?)

(define (html-tag? tag-name)
  (memq tag-name html-tags))

(define html-tags
  '(html
    head
    link
    meta
    style
    title
    body
    h1 h2 h3 h4 h5 h6
    section
    div
    span
    hr
    li
    ol
    p
    pre
    ul
    a
    br
    cite
    code
    em))
