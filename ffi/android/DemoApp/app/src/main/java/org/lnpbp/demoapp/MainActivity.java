package org.lnpbp.demoapp;

import android.os.Bundle;

import com.google.android.material.floatingactionbutton.FloatingActionButton;
import com.google.android.material.snackbar.Snackbar;

import androidx.appcompat.app.AppCompatActivity;
import androidx.appcompat.widget.Toolbar;

import android.util.Log;
import android.view.View;
import android.view.Menu;
import android.view.MenuItem;

import org.lnpbp.rgbnode.COpaqueStruct;
import org.lnpbp.rgbnode.rgb_node;

public class MainActivity extends AppCompatActivity {

    @Override
    protected void onCreate(Bundle savedInstanceState) {
        super.onCreate(savedInstanceState);
        setContentView(R.layout.activity_main);
        Toolbar toolbar = findViewById(R.id.toolbar);
        setSupportActionBar(toolbar);

        FloatingActionButton fab = findViewById(R.id.fab);
        fab.setOnClickListener(new View.OnClickListener() {
            @Override
            public void onClick(View view) {
                final COpaqueStruct runtime = ((DemoApp) getApplication()).runtime;
                try {
                    rgb_node.issue(runtime, "{\"network\":\"bitcoin\",\"ticker\":\"USDT\",\"name\":\"USD Tether\",\"issue_structure\":\"SingleIssue\",\"allocations\":[{\"coins\":100,\"vout\":0,\"txid\":\"0313ba7cfcaa66029a1a63918ebc426259f00953016c461663315d1bf6b83ab4\"}],\"precision\":0}");
                } catch (RuntimeException e) {
                    Log.e("RGB_NODE", e.getMessage());
                }
            }
        });
    }

    @Override
    public boolean onCreateOptionsMenu(Menu menu) {
        // Inflate the menu; this adds items to the action bar if it is present.
        getMenuInflater().inflate(R.menu.menu_main, menu);
        return true;
    }

    @Override
    public boolean onOptionsItemSelected(MenuItem item) {
        // Handle action bar item clicks here. The action bar will
        // automatically handle clicks on the Home/Up button, so long
        // as you specify a parent activity in AndroidManifest.xml.
        int id = item.getItemId();

        //noinspection SimplifiableIfStatement
        if (id == R.id.action_settings) {
            return true;
        }

        return super.onOptionsItemSelected(item);
    }
}
