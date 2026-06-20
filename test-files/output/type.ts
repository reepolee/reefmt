// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

export type ProcessOptions = {
	/** 
	 * Supported output formats and their MIME types. 
	 * */
	crop?: {
		left: number; // my comment
		top: number;
		width: number;
		height: number;
	};
	resize?: { width: number; height: number; };
	format?: string;
	quality?: number;
	delete_original?: boolean;
};
