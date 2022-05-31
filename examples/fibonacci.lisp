(define N 10)

(define SCALE 2)
(define SHAPE (colorize
    (rotate (polygon N) (* PI 1.3))
    (interpolate (srgb 39 181 159) (darken (srgb 59 162 194) 0.5) 3)
))

;; Returns the fibonacci sequence modulo m
(defun fib_mod (n m) (let
    ((fib_sub (lambda (i a b)
        (if (== i 0) '() (cons (% a m) (fib_sub (- i 1) b (% (+ a b) m))))
    )))
    (fib_sub n 0 1)
))

(define fib_list (fib_mod (+ (* N N) 1) N))

(rand-advance-rule
    (matrix-choice2 N (lambda (i j)
        (if
            (or
                (some2 (lambda (x y) (and (== x i) (== y j))) fib_list)
                (some2 (lambda (x y) (and (== y i) (== x j))) fib_list)
            )
            1.0
            0.01
        )
    ))
    0.8
    0.2
)
