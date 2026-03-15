export default function StatusIndicator({ connected }: { connected: boolean }) {
  return (
    <span
      className="pulse-dot"
      style={{
        display: "inline-block",
        width: 8,
        height: 8,
        borderRadius: "50%",
        background: connected ? "var(--green)" : "var(--red)",
        boxShadow: connected
          ? "0 0 6px rgba(40, 205, 65, 0.5)"
          : "0 0 6px rgba(255, 59, 48, 0.4)",
      }}
    />
  );
}
