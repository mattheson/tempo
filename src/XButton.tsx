export function XButton({ onClick }: { onClick: () => void }) {
  return (
    <button
      onClick={onClick}
      className="absolute top-2 right-2 w-6 h-6 rounded-full bg-gray-200 flex items-center justify-center hover:bg-gray-300"
    >
      <span className="text-gray-600">&times;</span>
    </button>
  );
}
