export function greet(name: string): string {
  return `Hello, ${name}!`;
}

export interface ApiResponse {
  message: string;
  timestamp: number;
}
