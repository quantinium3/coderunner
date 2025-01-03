package comphubtypes

import "time"

type ExecutionRes struct {
	Success bool
	Error   string
	Output  string
}

type CompilationReq struct {
	Code     string   `json:"code" validate:"required"`
	Language string   `json:"language" validate:"required,oneof=js ts py go java rs kt cpp c cs"`
	StdInput []string `json:"stdinput,omitempty"`
}

type CompilationRes struct {
	Success   bool      `json:"success"`
	Output    string    `json:"output,omitempty"`
	Error     string    `json:"error,omitempty"`
	Timestamp time.Time `json:"timestamp"`
}
