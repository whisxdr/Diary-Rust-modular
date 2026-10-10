import { z } from 'zod';

/** The password rule is the same on both sides; the backend repeats it. */
export const passwordSchema = z
  .string()
  .min(12, 'Use at least 12 characters')
  .max(200, 'That password is too long');

export const registerSchema = z.object({
  email: z.string().email('Enter a valid email address'),
  display_name: z.string().trim().min(2, 'Use at least 2 characters').max(60),
  password: passwordSchema,
});

export const loginSchema = z.object({
  email: z.string().min(1, 'Email is required'),
  password: z.string().min(1, 'Password is required'),
});

export const changePasswordSchema = z
  .object({
    current_password: z.string().min(1, 'Enter your current password'),
    new_password: passwordSchema,
  })
  .refine((values) => values.current_password !== values.new_password, {
    message: 'The new password must be different',
    path: ['new_password'],
  });

export type RegisterInput = z.infer<typeof registerSchema>;
export type LoginInput = z.infer<typeof loginSchema>;

/** Turns a zod failure into `field: message` pairs for form display. */
export function fieldErrors(error: z.ZodError): Record<string, string> {
  const result: Record<string, string> = {};
  for (const issue of error.issues) {
    const key = issue.path.join('.') || 'form';
    result[key] ??= issue.message;
  }
  return result;
}