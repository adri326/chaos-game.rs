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

;; (print (interpolate (srgb 39 181 159) (srgb 255 255 255) 3))

(define SHAPE (colorize
    (polygon 6)
    (interpolate (srgb 39 181 159) (darken (srgb 39 159 159) 0.5) 3)
))

;; (or-rule
;;     0.4
;;     (advance-rule 0.5 0.5 (avoid2-choice 0 0))
;;     (advance-rule (/ 2.0 3.0) 0.5 (avoid-choice 0))
;; )

(define SCALE 2)

(or-rule
    0.2
    (tensored-rule (discrete-spiral-rule
        (advance-rule (/ 4.0 3.0) 0.75 (avoid-choice 0))
        0.2
        (+ (/ PI 9.0) 0.01)
        0.75
        1.0
        0.5
    ))
    (tensor-rule
        (tensor-choice (choice) (avoid-choice 1))
        0.75
        (/ 2.0 3.0)
        0.75
        0.2
        F
    )
    0.6
)
