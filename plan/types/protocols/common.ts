export interface HttpAccountObject {
  user: string;
  pass: string;
}

export interface ReverseTagObject {
  tag?: string;
}

export interface FallbackObject {
  name?: string;
  alpn?: string;
  path?: string;
  dest?: number | string;
  xver?: number;
}
