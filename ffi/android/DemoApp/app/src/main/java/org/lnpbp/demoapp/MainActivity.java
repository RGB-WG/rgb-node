package org.lnpbp.demoapp;

import android.os.Bundle;

import com.google.android.material.floatingactionbutton.FloatingActionButton;

import androidx.appcompat.app.AppCompatActivity;
import androidx.appcompat.widget.Toolbar;

import android.util.Log;
import android.view.View;
import android.view.Menu;
import android.view.MenuItem;

import org.lnpbp.rgbnode.Runtime;
import org.lnpbp.rgbnode.model.IssueArgs;

import java.util.ArrayList;
import java.util.Arrays;

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
                final Runtime runtime = ((DemoApp) getApplication()).getRuntime();
                try {
                    final IssueArgs.CoinAllocation allocation = new IssueArgs.CoinAllocation(100L, 0, "0313ba7cfcaa66029a1a63918ebc426259f00953016c461663315d1bf6b83ab4");
                    runtime.issue("bitcoin", "USDT", "USD Tether", null, "SingleIssue", Arrays.asList(allocation), 8, new ArrayList(), null);
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
