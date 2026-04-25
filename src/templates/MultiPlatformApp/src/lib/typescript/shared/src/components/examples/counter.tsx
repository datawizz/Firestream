import { useExampleStore } from '../../stores/example.store';
import { Button } from '../ui/button';

export function Counter() {
  const { count, increment, decrement, reset } = useExampleStore();

  return (
    <div className="p-6 border rounded-lg space-y-4">
      <h2 className="text-2xl font-semibold">Counter Example</h2>
      <p className="text-4xl font-bold">{count}</p>
      <div className="flex gap-2">
        <Button onClick={increment}>Increment</Button>
        <Button onClick={decrement} variant="outline">
          Decrement
        </Button>
        <Button onClick={reset} variant="ghost">
          Reset
        </Button>
      </div>
    </div>
  );
}
