# Change of variable in probability distribution

- 1D
  - Let $Y = h(X)$, and suppose $f$ is monotonic
    - $dx \cdot f_X(x) = |dy| \cdot f_Y(y)$ [Why is that? The mapping is bijective, so even after transformation, we are still looking at the same "event"] (need absolute sign because the function might be monotonic decreasing)
    - $f_X(x) = |h'(x)| \cdot f_Y(y) = |h'(x)| \cdot f_Y(h(x))$ [Note that $dy = h(x + dx) - h(x)$]
    - $|(h^{-1})'(y)| \cdot f_X(x) = |(h^{-1})'(y)| \cdot f_X(h^{-1}(y)) = f_Y(y)$ [Note that $dx = h^{-1}(y + dy) - h^{-1}(y)$]
      - Note that $(h^{-1})'(y) = 1/h'(x) = 1/h'(h^{-1}(y))$
      - how to proof? let $y = h(x)$, $1 = h'(x) \cdot dx/dy$, $dx/dy = 1/h'(x)$, $d(h^{-1}(y))/dy = 1/h'(h^{-1}(y))$
    - What about CDF?
      - $F_X(x) = F_Y(h(x))$
      - Two ways to understand it
        - first, both $F_X(x)$ and $F_Y(h(x))$ refer to the same "event," as $h(x)$ is bijective, so they are equal
        - second, $f_X(x) = h'(x)f_Y(y)$, $f_Y(y) = f_X(x)/h'(x)$. Then, $F_Y(y) = \int_{-\infty}^{y} f_Y(y)\,dy = \int_{-\infty}^{h^{-1}(h(x))} f_Y(h(x))h'(x)\,dx$ [change of variable for integral] $= \int_{-\infty}^{x} \frac{f_X(x)}{h'(x)}h'(x)\,dx = F_X(x)$.
        - third, $F_X(x) = \int_{-\infty}^{x} f_X(x)\,dx = \int_{-\infty}^{y} f_Y(y)\,dy = F_Y(y)$
        - fourth (just for fun, from $F_X$ to $F_X$ using change of variable), $F_X(x) = \int_{-\infty}^{x} f_X(x)\,dx = \int_{-\infty}^{h(x)} \frac{f_X(h^{-1}(y))}{h'(x)}\,dy$. Now, we know that $(h^{-1})'(y) = 1/h'(x)$, so $\int_{-\infty}^{h(x)} \frac{f_X(h^{-1}(y))}{h'(x)}\,dx = F_X(h^{-1}(h(x)))$ [by chain rule] $= F_X(x)$.
- 2D
  - Let $Z_1 = h_1(X_1, X_2)$, and $Z_2 = h_2(X_1, X_2)$, and $h_1$ and $h_2$ are monotonic
    - we can define the original square with matrix $\begin{bmatrix} dx_1 & 0 \\ 0 & dx_2 \end{bmatrix}$, let the referencing point be $(x_1, x_2)$
    - After transformation, the square becomes parallelogram $A = \begin{bmatrix} dz_{11} & dz_{12} \\ dz_{21} & dz_{22} \end{bmatrix}$, at the referencing point $(h_1(x_1, x_2), h_2(x_1, x_2))$
      - $dz_{11} = h_1(x_1 + dx_1, x_2) - h_1(x_1, x_2)$ [graphically, $\begin{bmatrix} dz_{11} & dz_{21} \end{bmatrix}$ is the difference of two positional vector, or the vector that is tangent to the transformed area]
      - $dz_{21} = h_2(x_1 + dx_1, x_2) - h_2(x_1, x_2)$
      - $dz_{12} = h_1(x_1, x_2 + dx_2) - h_1(x_1, x_2)$
      - $dz_{22} = h_2(x_1, x_2 + dx_2) - h_2(x_1, x_2)$
    - $dx_1 dx_2 \cdot f_X(x_1, x_2) = |A| \cdot f_Z(z_1, z_2)$
    - $|J| = |A|/(dx_1 dx_2)$, so $J = \begin{bmatrix} dz_{11}/dx_1 & dz_{12}/dx_2 \\ dz_{21}/dx_1 & dz_{22}/dx_2 \end{bmatrix} = \begin{bmatrix} h_{1,x_1} & h_{1,x_2} \\ h_{2,x_1} & h_{2,x_2} \end{bmatrix}$
    - $f_X(x_1, x_2) = |J| \cdot f_Z(z_1, z_2)$
    - $f_X(x_1, x_2)/|J| = f_Z(z_1, z_2)$ or $f_X(x_1, x_2) \cdot |J^{-1}| = f_Z(z_1, z_2)$ [Note that $|XX^{-1}| = |I| = 1$, and $|XX^{-1}| = |X||X^{-1}|$, so $|X^{-1}| = 1/|X|$]

# Change of variable in integral

- 1D
  - if $x = f(x')$
  - $dx = f'(x')dx'$
- 2D
  - Let $x_1 = h_1(u_1, u_2)$, and $x_2 = h_2(u_1, u_2)$. Our goal is to change from $x_1$ and $x_2$ to $u_1$ and $u_2$
  - Imagine in the $uv$ plane, we have a square with bottom left coordinate be $(u_{10}, u_{20})$ and two vectors $\begin{bmatrix} du_1 & 0 \\ 0 & du_2 \end{bmatrix}$, so that the volumne is $|\begin{bmatrix} du_1 & 0 \\ 0 & du_2 \end{bmatrix}| = du_1 du_2$
  - After transforming this square to $x_1 x_2$-coordinate, it becomes a parallelogram
    - the bottom left coordinate becomes $(h_1(u_{10}, u_{20}), h_2(u_{10}, u_{20}))$
    - the two tangent vectors are $A = \begin{bmatrix} h_1(u_{10} + du_1, u_{20}) - h_1(u_{10}, u_{20}) & h_2(u_{10} + du_1, u_{20}) - h_2(u_{10}, u_{20}) \\ h_1(u_{10}, u_{20} + du_2) - h_1(u_{10}, u_{20}) & h_2(u_{10}, u_{20} + du_2) - h_2(u_{10}, u_{20}) \end{bmatrix}$
    - the volume of the parallelogram can be calculated by determinant or cross product (as $a \times b = \|a\| \|b\| \sin\theta$)
      - $|A|$ is the volume, and we can write $|A|/(du_1 du_2) \cdot du_1 du_2$, and since determinant is linear in each row, $|A|/(du_1 du_2) = |J|$, and $|A| = |J| \cdot du_1 du_2$
    - thus, the parallelogram in $x_1 x_2$-plane, is approximately $|J| \cdot du_1 du_2$.
  - In conclusion, $\iint f(x_1, x_2)\,dx_1 dx_2 = \iint f(h_1(u_1, u_2), h_2(u_1, u_2))|J|\,du_1 du_2$
  - how to connect change of var for intgral to change of var for pdf?
    - say the original integral and integrand is $\iint f(x_1, x_2)\,dx_1 dx_2$ and $f(x_1, x_2)$. Our goal is find the new integrand $g(u_1, u_2)$ (or a "density" function) for new variable $u_1\ u_2$ such that $g(u_1, u_2)\,du_1 du_2 = f(x_1, x_2)\,dx_1 dx_2$. Similar as what we did before for pdf, $g(u_1, u_2) = f(x_1, x_2)|J| = f(h_1(u_1, u_2), h_2(u_1, u_2))|J|$. As you see here, new integrand is not simply $f(h_1(u_1, u_2), h_2(u_1, u_2))$, as this stuff will integrate to something different then the integration of $f(x, y)$
