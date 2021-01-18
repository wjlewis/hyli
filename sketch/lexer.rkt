#lang racket

(require "tree.rkt")

(provide process-scheme)

(struct LParen () #:transparent)
(struct RParen () #:transparent)
(struct Space (n) #:transparent)
(struct Newline () #:transparent)
(struct Sym (str) #:transparent)
(struct Num (str) #:transparent)

(define (process-scheme input)
  (map to-tree (tokenize input)))

(define (to-tree token)
  (match token
    [(Newline) (Inner 'br '() '())]
    [(Space n) (Inner 'span
                      '((class . whitespace))
                      (map (lambda (_) (Text "&nbsp;"))
                           (range n)))]
    [(LParen) (Inner 'span
                     '((class . delim))
                     (list (Text "(")))]
    [(RParen) (Inner 'span
                     '((class . delim))
                     (list (Text ")")))]
    [(Sym s) (Inner 'span
                    '((class . symbol))
                    (list (Text s)))]
    [(Num n) (Inner 'span
                    '((class . number))
                    (list (Text n)))]))

(define (tokenize input)
  (let loop ([tokens '()] [cs (string->list input)])
    (if (empty? cs)
        (reverse tokens)
        (match (lexer cs)
          [`(,t . ,cs)
           (loop (cons t tokens) cs)]))))

(define (lexer cs)
  (match cs
    [`(#\( ,cs ...) (cons (LParen) cs)]
    [`(#\) ,cs ...) (cons (RParen) cs)]
    [`(#\newline ,cs ...) (cons (Newline) cs)]
    [`(,c ,cs ...)
     #:when (whitespace? c)
     (lex-whitespace 1 cs)]
    [`(,c ,cs ...)
     #:when (symbol-start? c)
     (lex-symbol (list c) cs)]
    [`(,c ,cs ...)
     #:when (digit? c)
     (lex-number (list c) cs)]
    [else '()]))

(define (lex-whitespace n cs)
  (match cs
    [`(,c ,cs1 ...)
     #:when (whitespace? c)
     (lex-whitespace (add1 n) cs1)]
    [else (cons (Space n) cs)]))

(define (lex-symbol acc cs)
  (match cs
    [`(,c ,cs1 ...)
     #:when (symbol-follow? c)
     (lex-symbol (cons c acc) cs1)]
    [else (cons (Sym (list->string (reverse acc)))
                cs)]))

(define (lex-number acc cs)
  (match cs
    [`(,c ,cs1 ...)
     #:when (digit? c)
     (lex-number (cons c acc) cs1)]
    [else (cons (Num (list->string (reverse acc)))
                cs)]))

(define (whitespace? c)
  (eqv? c #\space))

(define (symbol-start? c)
  (memv c (string->list
           (string-append "abcdefghijklmnopqrstuvwxyz"
                          "ABCDEFGHIJKLMNOPQRSTUVWXYZ"
                          "!$%^&*-_=+:<>/?"))))

(define (symbol-follow? c)
  (or (symbol-start? c)
      (digit? c)))

(define (digit? c)
  (memv c (string->list "0123456789")))
