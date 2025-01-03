package compilers

import (
	"encoding/json"
	"fmt"
	"log"
	"net/http"
	"time"

	"github.com/go-playground/validator/v10"
	comphubtypes "github.com/quantinium03/comphub/internal/comphubTypes"
	"github.com/quantinium03/comphub/internal/execute"
	"github.com/quantinium03/comphub/internal/utils"
)

func CompileProgram(w http.ResponseWriter, r *http.Request) {
	var req comphubtypes.CompilationReq

	defer r.Body.Close()
	if err := json.NewDecoder(r.Body).Decode(&req); err != nil {
		log.Printf("Parsing Error: %v", err)
		res := comphubtypes.CompilationRes{
			Success:   false,
			Error:     fmt.Sprintf("Parsing Error: %v", err),
			Timestamp: time.Now(),
		}
		writeJSONResponse(w, http.StatusBadRequest, res)
		return
	}

	validate := validator.New()
	if err := validate.Struct(req); err != nil {
		var validationError string

		if err.Error() == "Key: 'CompRequest.Language' Error:Field validation for 'Language' failed on the 'eq=js|eq=ts|eq=py|eq=go' tag" {
			validationError = "Provide us with a valid language extension..."
		} else {
			validationError = err.Error()
		}

		log.Printf("Validation Error: %s", validationError)
		res := comphubtypes.CompilationRes{
			Success:   false,
			Error:     fmt.Sprintf("Validation Error: %s", validationError),
			Timestamp: time.Now(),
		}
		writeJSONResponse(w, http.StatusBadRequest, res)
		return
	}

	createFileRes := utils.CreateFile(req.Code, req.Language)
	if !createFileRes.Success {
		log.Printf("Error creating file: %v", createFileRes.Error)
		res := comphubtypes.CompilationRes{
			Success:   false,
			Error:     fmt.Sprintf("Error in creating file: %v", createFileRes.Error),
			Timestamp: time.Now(),
		}
		writeJSONResponse(w, http.StatusBadRequest, res)
		return
	}

	log.Printf("Successfully created file: %s (%s)", createFileRes.Filename, req.Language)

	var stdout, stderr string

	switch req.Language {
	case "js":
		compileRes, err := execute.ExecuteJS(createFileRes.Filename, req.StdInput)
		if err != nil {
			stderr = compileRes.Error
		} else {
			stdout = compileRes.Output
		}
		break
	case "c":
		compileRes, err := execute.ExecuteC(createFileRes.Filename, req.StdInput)
		if err != nil {
			stderr = compileRes.Error
		} else {
			stdout = compileRes.Output
		}

	case "cpp":
		compileRes, err := execute.ExecuteCPP(createFileRes.Filename, req.StdInput)
		if err != nil {
			stderr = compileRes.Error
		} else {
			stdout = compileRes.Output
		}

	case "go":
		compileRes, err := execute.ExecuteGo(createFileRes.Filename, req.StdInput)
		if err != nil {
			stderr = compileRes.Error
		} else {
			stdout = compileRes.Output
		}
	case "java":
		compileRes, err := execute.ExecuteJava(createFileRes.Filename, req.StdInput)
		if err != nil {
			stderr = compileRes.Error
		} else {
			stdout = compileRes.Error
		}
	}

	if stderr != "" {
		res := comphubtypes.CompilationRes{
			Success:   false,
			Error:     fmt.Sprintf("Execution error: ", stderr),
			Timestamp: time.Now(),
		}
		writeJSONResponse(w, http.StatusBadRequest, res)
		return
	}

	res := comphubtypes.CompilationRes{
		Success:   true,
		Output:    stdout,
		Timestamp: time.Now(),
	}

	utils.DeleteFile(createFileRes.Filename)
	writeJSONResponse(w, http.StatusOK, res)
}

func writeJSONResponse(w http.ResponseWriter, status int, data interface{}) {
	w.Header().Set("Content-Type", "application/json")
	w.WriteHeader(status)
	json.NewEncoder(w).Encode(data)
}
