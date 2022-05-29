(define PHI (/ (+ (sqrt 5.0) 1.0) 2.0))
(define PI 3.14159265358979323846264338327950288)

(define polygon (lambda (n) (map
    (lambda (i) (let ((x (* (* 2.0 PI) (/ (float i) (float n)))))
        (list (cos x) (sin x))
    ))
    (range 0 n)
)))

(define SHAPE (polygon 6))

;; (or-rule
;;     0.4
;;     (advance-rule 0.5 0.5 (avoid2-choice 0 0))
;;     (advance-rule (/ 2.0 3.0) 0.5 (avoid-choice 0))
;; )

(define SCALE 2)

(or-rule
    0.2
    (tensored-rule (discrete-spiral-rule
        (advance-rule (/ 4.0 3.0) 0.5 (avoid-choice 0))
        0.2
        (/ PI 9.0)
        0.75
        1.0
        0.5
    ))
    (tensor-rule
        (tensor-choice (choice) (avoid-choice 1))
        0.75
        (/ 2.0 3.0)
        0.5
        0.2
        F
    )
    0.6
)
