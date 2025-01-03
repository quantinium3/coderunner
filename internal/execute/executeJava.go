package execute

import (
	"context"
	"fmt"
	"io"
	"os/exec"
	"path/filepath"
	"strings"
	"time"

	comphubtypes "github.com/quantinium03/comphub/internal/comphubTypes"
)

func ExecuteJava(filename string, stdinputs []string) (comphubtypes.ExecutionRes, error) {
	ctx, cancel := context.WithTimeout(context.Background(), 15*time.Second)
	defer cancel()

	// Compile the Java file
	compileCmd := exec.CommandContext(ctx, "javac", filename)
	compileOutput, compileErr := compileCmd.CombinedOutput()
	if compileErr != nil {
		return comphubtypes.ExecutionRes{
			Success: false,
			Error:   fmt.Sprintf("Compilation failed: %s", string(compileOutput)),
		}, compileErr
	}

	// Extract the class name (filename without path and ".java" extension)
	className := filepath.Base(strings.TrimSuffix(filename, ".java"))

	// Run the compiled Java class
	runCmd := exec.CommandContext(ctx, "java", className)
	runCmd.Dir = filepath.Dir(filename) // Set working directory

	input, err := runCmd.StdinPipe()
	if err != nil {
		return comphubtypes.ExecutionRes{
			Success: false,
			Error:   fmt.Sprintf("Failed to create the stdin pipe: %v", err),
		}, err
	}

	output, err := runCmd.StdoutPipe()
	if err != nil {
		return comphubtypes.ExecutionRes{
			Success: false,
			Error:   fmt.Sprintf("Failed to create the stdout pipe: %v", err),
		}, err
	}

	stderr, err := runCmd.StderrPipe()
	if err != nil {
		return comphubtypes.ExecutionRes{
			Success: false,
			Error:   fmt.Sprintf("Failed to create the stderr pipe: %v", err),
		}, err
	}

	if err := runCmd.Start(); err != nil {
		return comphubtypes.ExecutionRes{
			Success: false,
			Error:   fmt.Sprintf("Failed to start the command: %v", err),
		}, err
	}

	// Write standard input
	go func() {
		defer input.Close()
		for _, stdinput := range stdinputs {
			if _, err := input.Write([]byte(stdinput + "\n")); err != nil {
				cancel()
				return
			}
		}
	}()

	// Read standard output and error
	stdoutBytes, err := io.ReadAll(output)
	if err != nil {
		return comphubtypes.ExecutionRes{
			Success: false,
			Error:   fmt.Sprintf("Failed to read stdout: %v", err),
		}, err
	}

	stderrBytes, err := io.ReadAll(stderr)
	if err != nil {
		return comphubtypes.ExecutionRes{
			Success: false,
			Error:   fmt.Sprintf("Failed to read stderr: %v", err),
		}, err
	}

	if err := runCmd.Wait(); err != nil {
		return comphubtypes.ExecutionRes{
			Success: false,
			Error:   fmt.Sprintf("Command execution failed: %v", err),
		}, err
	}

	if len(stderrBytes) > 0 {
		return comphubtypes.ExecutionRes{
			Success: false,
			Error:   string(stderrBytes),
		}, fmt.Errorf("stderr: %v", stderrBytes)
	}

	return comphubtypes.ExecutionRes{
		Success: true,
		Output:  string(stdoutBytes),
	}, nil
}
