(define SHAPE (polygon 6))

(advance-rule
    (avoid-choice 0) ;; Randomly choose a point, but avoid the previous point
    (- PHI 1.0) ;; Move by (φ-1) ≈ 0.618033 the distance towards it
)
