export type State='created'|'resolving'|'queued'|'connecting'|'downloading'|'pausing'|'paused'|'retry_waiting'|'merging'|'verifying'|'completed'|'failed'|'cancelled';
export interface Download {id:string;url:string;filename:string;destination:string;state:State;queue_position:number;total_bytes:number|null;downloaded_bytes:number;connection_count:number;error:string|null}
export interface GlobalStatus {downloaded_bytes:number;known_total_bytes:number;percentage:number|null;combined_speed:number;active:number;queued:number;paused:number;completed:number;failed:number}
