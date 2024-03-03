export interface Range {
    sourceIndex: number,
    startLine: number,
    startPos: number,
    endLine: number,
    endPos: number,
}
export interface DateResult {
    formatted: string,
}

export interface NumberResult {
    fmtd: string,
    u: string,
}

export interface ErrorResult {
    msg: string,
    type: "E" | "W",
    range: Range,
    stackTrace: ErrorResult[],
}
export interface DurationResult {
    years: number,
    months: number,
    days: number,
}

export interface ResultLine {
    type: string,
    src: number,
    line: number,
    id: string,
    date: DateResult,
    duration: DurationResult,
    comment: string,
    number: NumberResult,
    list: ResultLine[],
}

export interface Results {
    result: ResultLine[],
    errors: ErrorResult[],
}