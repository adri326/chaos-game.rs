(define PHI (/ (+ (sqrt 5.0) 1.0) 2.0))
(define PI 3.14159265358979323846264338327950288)

(defun map2-sub (fn l n) (if (== l '()) '() (cons (fn (car l) n) (map2-sub fn (cdr l) (+ n 1)))))
(defun map2 (fn l) (map2-sub fn l 0))

(define colorize (lambda (l c) (map2
    (lambda (x i)
        (let ((c2 (nth (% i (length c)) c))) ; The current color
        (list (nth 0 x) (nth 1 x) (nth 0 c2) (nth 1 c2) (nth 2 c2))
    ))
    l
)))

(define polygon (lambda (n) (map
    (lambda (i) (let ((x (* (* 2.0 PI) (/ (float i) (float n)))))
        (list (cos x) (sin x))
    ))
    (range 0 n)
)))

(defun rotate (l a) (let ((a (float a))) (map2
    (lambda (p _) (if (== (length p) 2)
        (list
            (+ (* (cos a) (float (nth 0 p))) (* (sin a) (float (nth 1 p))))
            (- (* (cos a) (float (nth 1 p))) (* (sin a) (float (nth 0 p))))
        )
        (list
            (+ (* (cos a) (float (nth 0 p))) (* (sin a) (float (nth 1 p))))
            (- (* (cos a) (float (nth 1 p))) (* (sin a) (float (nth 0 p))))
            (nth 2 p)
            (nth 3 p)
            (nth 4 p)
        )
    ))
    l
)))

(define srgb (lambda (r g b) (let ((GAMMA 2.2)) (list
    (pow (/ (float r) 255.0) GAMMA)
    (pow (/ (float g) 255.0) GAMMA)
    (pow (/ (float b) 255.0) GAMMA)
))))

(define interpolate (lambda (a b n) (map
    (lambda (i) (let
        ((x (/ (float i) (- (float n) 1.0)))) ; Ratio between a and b
        (list
            (+ (* (nth 0 a) (- 1.0 x)) (* (nth 0 b) x))
            (+ (* (nth 1 a) (- 1.0 x)) (* (nth 1 b) x))
            (+ (* (nth 2 a) (- 1.0 x)) (* (nth 2 b) x))
        )
    ))
    (range 0 n)
)))

(define darken (lambda (c a) (map (lambda (x) (* x a)) c)))

(define matrix-choice2 (lambda (n fn) (matrix-choice
    n
    (map
        (lambda (i) (float (fn (% i n) (/ i n))))
        (range 0 (* n n))
    )
)))


;; (or-rule
;;     0.2
;;     (tensored-rule (discrete-spiral-rule
;;         (advance-rule (/ 4.0 3.0) 0.75 (avoid-choice 0))
;;         0.2
;;         (+ (/ PI 9.0) 0.01)
;;         0.75
;;         1.0
;;         0.5
;;     ))
;;     (tensor-rule
;;         (tensor-choice (choice) (avoid-choice 1))
;;         0.75
;;         (/ 2.0 3.0)
;;         0.75
;;         0.2
;;         F
;;     )
;;     0.6
;; )

;; Returns true iff a % n == b % n, ie a === b [mod n]
(defun modeq (a b n) (== (% a n) (% b n)))

(define N 5)

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
    (fib_sub n 0 6)
))

;; (define fib_list (fib_mod (+ (* N N) 1) N))

(defun find2 (fn l)
    (if (<= (length l) 1)
        F
        (or
            (fn (car l) (nth 1 l))
            (find2 fn (cdr l))
        )
    )
)

;; (or-rule
;;     0.1
;;     (discrete-spiral-rule
;;         (advance-rule (/ 4.0 3.0) 0.5 (matrix-choice2 N (lambda (i j)
;;             (find2 (lambda (x y) (and (== x i) (== y j))) fib_list)
;;         )))
;;         0.5
;;         PI
;;         (/ 1.0 3.0)
;;     )
;;     (advance-rule (/ 2.0 3.0) 0.5 (matrix-choice2 N (lambda (i j)
;;         (find2 (lambda (x y) (and (== x i) (== y j))) fib_list)
;;     )))
;;     0.5
;; )

;; (define my-choice (matrix-choice2 N (lambda (i j)
;;         (if
;;             (or
;;                 (find2 (lambda (x y) (and (== x i) (== y j))) fib_list)
;;                 (find2 (lambda (x y) (and (== y i) (== x j))) fib_list)
;;             )
;;             1.0
;;             0.01
;;         )
;;     ))
;; )

(or-rule
    0.1
    (darken-rule
        (rand-advance-rule
            (neighborhood-choice 1)
            PHI
            0.1
            2.0
        )
        0.5
    )
    (advance-rule
        (avoid2-choice 0 0)
        (- PHI 1.0)
        0.66
    )
    0.4
)
