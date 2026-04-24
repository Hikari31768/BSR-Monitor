export function formatEpochTime(epoch: number): string {
  const date = new Date(epoch * 1000);
  return date.toLocaleDateString('zh-CN', {
    year: 'numeric',
    month: '2-digit',
    day: '2-digit',
    hour: '2-digit',
    minute: '2-digit',
  });
}

export function formatAccuracy(accuracy: number): string {
  return `${(accuracy * 100).toFixed(2)}%`;
}

export function formatRating(rating?: number): string {
  if (rating == null) return '未评分';
  return `${(rating * 100).toFixed(1)}%`;
}