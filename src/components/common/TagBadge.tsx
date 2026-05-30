import { getTagColor } from '../../utils/format';

interface TagBadgeProps {
  tag: string;
  onRemove?: () => void;
}

export function TagBadge({ tag, onRemove }: TagBadgeProps) {
  return (
    <span className={`inline-flex items-center gap-1 px-2 py-0.5 rounded-full text-xs font-medium ${getTagColor(tag)}`}>
      {tag}
      {onRemove && (
        <button onClick={onRemove} className="ml-0.5 hover:opacity-70">×</button>
      )}
    </span>
  );
}
