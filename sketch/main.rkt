#lang racket

(require "tree.rkt")
(require "tags.rkt")
(require "lexer.rkt")

(define (transform tree transforms)
  (match tree
    [(Inner t bs cs)
     (if (html-tag? t)
         (Inner t bs (map (lambda (child)
                            (transform child transforms))
                          cs))
         (let ([trans-fn (hash-ref transforms t)])
           (transform (trans-fn bs cs) transforms)))]
    [_ tree]))

;; Testing
;; Here's the document that we're going to transform:
(define tree
  (parse-tree
   '(Doc ((title . "My first document"))
         (Section ((ref . sec-1))
                  "This is a first section"
                  "With a couple of paragraphs")
         (Section ()
                  (Listing.Scheme ()
                                  "(define x 42)
(define (fact n)
  (if (zero? n)
      1
      (* n (fact (sub1 n)))))")))))

;; Transformer definitions:
(define (Doc bindings children)
  (let ([title (assq 'title bindings)])
    (Inner 'html
           '()
           (list (Inner 'Head
                        `(,title)
                        '())
                 (Inner 'body
                        '()
                        children)))))

(define (Head bindings children)
  (let ([title (assq 'title bindings)])
    (Inner 'head
           '()
           (list (Inner 'meta
                        '((charset . utf-8))
                        '())
                 (Inner 'title
                        '()
                        (list (Text (cdr title))))))))

(define (Section bindings children)
  (let ([ref (assq 'ref bindings)])
    (let ([bindings (if ref `((id . ,(cdr ref))) '())])
      (Inner 'section
             bindings
             (map (lambda (child)
                    (Inner 'p
                           '()
                           (list child)))
                  children)))))

(define (Listing.Scheme bindings children)
  (match children
    [`(,(Text code))
     (Inner 'code
            '()
            (process-scheme code))]
    [else (error
           (format "Expected a single text node, not ~a."
                   children))]))

(define transforms
  (make-immutable-hash `((Doc . ,Doc)
                         (Head . ,Head)
                         (Section . ,Section)
                         (Listing.Scheme . ,Listing.Scheme))))


(define transformed
  (transform tree transforms))

;; Write `transformed` to a file
(define out (open-output-file "out.html" #:exists 'replace))
(write-string (format "~v" transformed) out)
(close-output-port out)
