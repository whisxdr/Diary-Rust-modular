import type { ButtonHTMLAttributes, ReactNode } from 'react';

type Variant = 'primary' | 'secondary' | 'ghost' | 'danger';

interface ButtonProps extends ButtonHTMLAttributes<HTMLButtonElement> {
  variant?: Variant;
  children: ReactNode;
}

export function Button({ variant = 'secondary', children, ...rest }: ButtonProps) {
  return (
    <button className={`button button--${variant}`} {...rest}>
      {children}
    </button>
  );
}