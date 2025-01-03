package execute

import (
	"context"
	"fmt"
	"io"
	"os/exec"
	"time"

	comphubtypes "github.com/quantinium03/comphub/internal/comphubTypes"
)

func ExecuteJS(filename string, stdinputs []string) (comphubtypes.ExecutionRes, error) {
	ctx, cancel := context.WithTimeout(context.Background(), 10*time.Second)
	defer cancel()

	cmd := exec.CommandContext(ctx, "node", "--no-warnings", filename)

	input, err := cmd.StdinPipe()
	if err != nil {
		return comphubtypes.ExecutionRes{
			Success: false,
			Error:   fmt.Sprintf("failed to create stdin pipe: %v", err),
		}, err
	}
	output, err := cmd.StdoutPipe()
	if err != nil {
		return comphubtypes.ExecutionRes{
			Success: false,
			Error:   fmt.Sprintf("failed to create stdout pipe: %v", err),
		}, err
	}
	stderr, err := cmd.StderrPipe()
	if err != nil {
		return comphubtypes.ExecutionRes{
			Success: false,
			Error:   fmt.Sprintf("failed to create stderr pipe: %v", err),
		}, err
	}

	if err := cmd.Start(); err != nil {
		return comphubtypes.ExecutionRes{
			Success: false,
			Error:   fmt.Sprintf("failed to start command: %v", err),
		}, err
	}

	go func() {
		defer input.Close()
		for _, stdinput := range stdinputs {
			if _, err := input.Write([]byte(stdinput + "\n")); err != nil {
				cancel()
				return
			}
		}
	}()

	stdoutBytes, err := io.ReadAll(output)
	if err != nil {
		return comphubtypes.ExecutionRes{
			Success: false,
			Error:   fmt.Sprintf("failed to read stdout: %v", err),
		}, err
	}

	stderrBytes, err := io.ReadAll(stderr)
	if err != nil {
		return comphubtypes.ExecutionRes{
			Success: false,
			Error:   fmt.Sprintf("failed to read stderr: %v", err),
		}, err
	}

	if err := cmd.Wait(); err != nil {
		return comphubtypes.ExecutionRes{
			Success: false,
			Error:   fmt.Sprintf("command execution failed: %v", err),
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
