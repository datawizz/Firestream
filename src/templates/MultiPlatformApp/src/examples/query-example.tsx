/**
 * TanStack Query Example
 *
 * Demonstrates how to use TanStack Query with platform adapters.
 */

import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { ServiceFactory } from '@multi-platform-app/shared/services';

interface User {
  id: string;
  name: string;
}

export function QueryExample() {
  const queryClient = useQueryClient();

  // Fetch user data
  const { data: user, isLoading } = useQuery({
    queryKey: ['user'],
    queryFn: async () => {
      const storage = await ServiceFactory.getStorageService();
      const data = await storage.getItem('user');
      return data ? JSON.parse(data) : null;
    },
  });

  // Mutation to update user
  const updateUser = useMutation({
    mutationFn: async (newUser: User) => {
      const storage = await ServiceFactory.getStorageService();
      await storage.setItem('user', JSON.stringify(newUser));
      return newUser;
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['user'] });
    },
  });

  if (isLoading) return <div>Loading...</div>;

  return (
    <div>
      <h3>User: {user?.name || 'No user'}</h3>
      <button
        onClick={() =>
          updateUser.mutate({ id: '1', name: 'John Doe' })
        }
      >
        Update User
      </button>
    </div>
  );
}
