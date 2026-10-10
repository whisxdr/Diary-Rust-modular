export function Spinner({ label = 'Loading' }: { label?: string }) {
  return (
    <p className="spinner" role="status">
      {label}…
    </p>
  );
}