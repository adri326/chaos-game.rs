(define N 6)

(define SCALE 2)
(define SHAPE (colorize
    (rotate (polygon N) (* PI 0.5))
    (interpolate (srgb 39 181 159) (darken (srgb 39 159 181) 0.5) 4)
))

;; Returns the fibonacci sequence modulo m
(defun fib_mod (n m) (let
    ((fib_sub (lambda (i a b)
        (if (== i 0) '() (cons (% a m) (fib_sub (- i 1) b (% (+ a b) m))))
    )))
    (fib_sub n 0 1)
))

(define fib_list (fib_mod (+ (* N N) 1) N))

(defun find2 (fn l)
    (if (<= (length l) 1)
        F
        (or
            (fn (car l) (nth 1 l))
            (find2 fn (cdr l))
        )
    )
)

(define my-choice (matrix-choice2 N (lambda (i j)
    (find2 (lambda (x y) (and (== x i) (== y j))) fib_list)
)))

(or-rule
    0.1
    (discrete-spiral-rule
        (advance-rule my-choice (/ 4.0 3.0) 0.5)
        0.5
        PI
        (/ 1.0 3.0)
    )
    (advance-rule my-choice (/ 2.0 3.0) 0.5)
    0.5
)
