(define PHI (/ (+ (sqrt 5.0) 1.0) 2.0))
(define PI 3.14159265358979323846264338327950288)

(define map2 (let
    ((map2-sub (lambda (fn l n)
        (if (== l '()) '() (cons (fn (car l) n) (map2-sub fn (cdr l) (+ n 1))))
    )))
    (lambda (fn l) (map2-sub fn l 0))
))

(defun colorize (l c) (map2
    (lambda (x i)
        (let ((c2 (nth (% i (length c)) c))) ; The current color
        (list (nth 0 x) (nth 1 x) (nth 0 c2) (nth 1 c2) (nth 2 c2))
    ))
    l
))

(defun polygon (n) (map
    (lambda (i) (let ((x (* (* 2.0 PI) (/ (float i) (float n)))))
        (list (cos x) (sin x))
    ))
    (range 0 n)
))

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

(defun srgb (r g b) (let ((GAMMA 2.2)) (list
    (pow (/ (float r) 255.0) GAMMA)
    (pow (/ (float g) 255.0) GAMMA)
    (pow (/ (float b) 255.0) GAMMA)
)))

(defun interpolate (a b n) (map
    (lambda (i) (let
        ((x (/ (float i) (- (float n) 1.0)))) ; Ratio between a and b
        (list
            (+ (* (nth 0 a) (- 1.0 x)) (* (nth 0 b) x))
            (+ (* (nth 1 a) (- 1.0 x)) (* (nth 1 b) x))
            (+ (* (nth 2 a) (- 1.0 x)) (* (nth 2 b) x))
        )
    ))
    (range 0 n)
))

(defun darken (c a) (map (lambda (x) (* x a)) c))

(defun matrix-choice2 (n fn) (matrix-choice
    n
    (map
        (lambda (i) (float (fn (% i n) (/ i n))))
        (range 0 (* n n))
    )
))

(defun subset-choice (l) (matrix-choice2 (length l) (lambda (i j) (nth i l))))

(define or-rules (let ((or-rules-sub (lambda (p l) (if (== (length l) 1)
        (car (car l))
        (or-rule
            (nth 1 (car l))
            (car (car l))
            (or-rules-sub (/ p (- 1.0 (nth 1 (car l)))) (cdr l))
        )
    ))))
    (lambda (l) (or-rules-sub 1.0 l))
))

;; Returns true if (fn x) returns true for some x = l[i]
(defun some (fn l)
    (if (is-null l) F
        (or
            (fn (car l))
            (find fn (cdr l))
        )
    )
)

;; Returns true if (fn x y) returns true for x = l[i] and y = l[i + 1]
(defun some2 (fn l)
    (if (<= (length l) 1)
        F
        (or
            (fn (car l) (nth 1 l))
            (find2 fn (cdr l))
        )
    )
)

;; Convenience alias
(define rand-advance-rule random-advance-rule)
